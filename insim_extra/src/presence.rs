//! [`Presence`] tracks connections and players from a bare `insim` packet
//! stream. Feed packets in, query state out.
//!
//! Admin commands return [`insim::Packet`] values for the caller to send;
//! multi-packet commands return [`Vec<insim::Packet>`].
//!
//! ```ignore
//! let presence = Presence::new();
//!
//! // In a bare insim loop:
//! while let Some(packet) = conn.next().await {
//!     // state-only, no events:
//!     presence.apply(&packet);
//!
//!     // or collect state-change events:
//!     for event in presence.apply_events(&packet) {
//!         match event {
//!             PresenceEvent::Connected(info) => println!("{} joined", info.pname),
//!             _ => {}
//!         }
//!     }
//! }
//! ```

use std::{
    collections::{HashMap, HashSet},
    net::Ipv4Addr,
    sync::{Arc, RwLock},
    time::Duration,
};

use insim::{
    core::vehicle::Vehicle,
    identifiers::{ConnectionId, PlayerId},
    insim::{
        Cnl, Cpr, Nci, Ncn, Npl, PenaltyInfo, Pfl, Pla, PlayerFlags, PlayerType, Pll, Slc, Tiny,
        TinyType, Toc,
    },
};
use tokio_util::sync::CancellationToken;

use crate::util::host_command;

/// Per-connection record stored by [`Presence`].
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Connection identifier.
    pub ucid: ConnectionId,
    /// LFS.net username.
    pub uname: String,
    /// Player nickname (display name).
    pub pname: String,
    /// Whether the connection has admin privileges.
    pub admin: bool,
    /// All players (including AI) controlled by this connection.
    pub players: HashSet<PlayerId>,
    /// LFS.net user ID. Only populated when the host has an admin password
    /// set and an `Nci` packet has been received for this connection.
    pub userid: Option<u32>,
    /// Originating IP address. Only populated on hosts with an admin
    /// password (via `Nci`).
    pub ipaddress: Option<Ipv4Addr>,
    /// Most recently selected vehicle in the garage (from `Slc`).
    pub selected_vehicle: Option<Vehicle>,
}

/// Per-player record stored by [`Presence`].
#[derive(Debug, Clone)]
pub struct PlayerInfo {
    /// Player ID.
    pub plid: PlayerId,
    /// Owning connection ID.
    pub ucid: ConnectionId,
    /// Vehicle in use.
    pub vehicle: Vehicle,
    /// Player type flags (AI, female, remote, etc.).
    pub ptype: PlayerType,
    /// Player flags (pit-stop done, swap-out allowed, etc.).
    pub flags: PlayerFlags,
    /// Whether the player is currently in the pit lane.
    pub in_pitlane: bool,
    /// Player nickname at the moment of join.
    pub pname: String,
}

/// State-change events produced by [`Presence::apply_events`].
#[derive(Debug, Clone)]
pub enum PresenceEvent {
    /// A new connection joined.
    Connected(ConnectionInfo),
    /// A connection left.
    Disconnected {
        /// The connection that left.
        ucid: ConnectionId,
        /// Last known info (cloned before removal).
        info: Option<ConnectionInfo>,
    },
    /// Extra connection details arrived via `Nci`.
    ConnectionDetails(ConnectionInfo),
    /// A connection selected a vehicle in the garage.
    VehicleSelected {
        /// The connection.
        ucid: ConnectionId,
        /// The selected vehicle.
        vehicle: Vehicle,
    },
    /// A connection changed their display name.
    Renamed {
        /// Connection ID.
        ucid: ConnectionId,
        /// Stable LFS.net username.
        uname: String,
        /// New display name.
        new_pname: String,
    },
    /// A player joined the track.
    PlayerJoined(PlayerInfo),
    /// A player left the track.
    PlayerLeft(PlayerInfo),
    /// A driver swap occurred.
    TakingOver {
        /// Player state before the swap.
        before: PlayerInfo,
        /// Player state after the swap.
        after: PlayerInfo,
    },
    /// A player tele-pitted (Shift+P).
    PlayerTeleportedToPits(PlayerInfo),
}

#[derive(Default)]
struct PresenceInner {
    connections: HashMap<ConnectionId, ConnectionInfo>,
    players: HashMap<PlayerId, PlayerInfo>,
    /// Survives `Cnl`: maps LFS.net username → last seen display name.
    last_known_names: HashMap<String, String>,
}

/// Tracks active connections and players from a stream of `insim` packets.
///
/// State lives behind `Arc<RwLock<…>>`; clones are cheap and share the same
/// maps.
///
/// Feed packets with [`apply`](Self::apply) (state-only) or
/// [`apply_events`](Self::apply_events) (state + change events). Admin commands
/// return packets for the caller to send.
#[derive(Clone, Default)]
pub struct Presence {
    inner: Arc<RwLock<PresenceInner>>,
}

impl std::fmt::Debug for Presence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Presence")
            .field("connections", &self.connection_count())
            .field("players", &self.player_count())
            .finish()
    }
}

impl Presence {
    /// Create a new presence tracker with empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of tracked connections.
    pub fn connection_count(&self) -> usize {
        self.inner.read().expect("poison").connections.len()
    }

    /// Alias for [`connection_count`](Self::connection_count).
    pub fn count(&self) -> usize {
        self.connection_count()
    }

    /// Number of tracked players.
    pub fn player_count(&self) -> usize {
        self.inner.read().expect("poison").players.len()
    }

    /// Look up one connection by UCID.
    pub fn get(&self, ucid: ConnectionId) -> Option<ConnectionInfo> {
        self.inner
            .read()
            .expect("poison")
            .connections
            .get(&ucid)
            .cloned()
    }

    /// Snapshot of all tracked connections.
    pub fn connections(&self) -> Vec<ConnectionInfo> {
        self.inner
            .read()
            .expect("poison")
            .connections
            .values()
            .cloned()
            .collect()
    }

    /// Look up one player by `PlayerId`.
    pub fn player(&self, plid: PlayerId) -> Option<PlayerInfo> {
        self.inner
            .read()
            .expect("poison")
            .players
            .get(&plid)
            .cloned()
    }

    /// Snapshot of all tracked players.
    pub fn players(&self) -> Vec<PlayerInfo> {
        self.inner
            .read()
            .expect("poison")
            .players
            .values()
            .cloned()
            .collect()
    }

    /// Look up the connection that currently controls a given player.
    pub fn connection_by_player(&self, plid: PlayerId) -> Option<ConnectionInfo> {
        let guard = self.inner.read().expect("poison");
        let player = guard.players.get(&plid)?;
        guard.connections.get(&player.ucid).cloned()
    }

    /// Last known display name for an LFS.net username. Survives disconnect.
    pub fn last_known_name(&self, uname: &str) -> Option<String> {
        self.inner
            .read()
            .expect("poison")
            .last_known_names
            .get(uname)
            .cloned()
    }

    /// Batch variant of [`last_known_name`](Self::last_known_name).
    pub fn last_known_names<I, S>(&self, unames: I) -> HashMap<String, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let guard = self.inner.read().expect("poison");
        unames
            .into_iter()
            .filter_map(|u| {
                let u = u.into();
                guard.last_known_names.get(&u).map(|p| (u, p.clone()))
            })
            .collect()
    }

    /// Returns a `/kick` packet for the given UCID, or `None` if not found.
    pub fn kick(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        let conn = self.get(ucid)?;
        Some(host_command(format!("/kick {}", conn.uname)))
    }

    /// Returns a `/ban` packet. `ban_days = 0` means 12 hours (LFS convention).
    pub fn ban(&self, ucid: ConnectionId, ban_days: u32) -> Option<insim::Packet> {
        let conn = self.get(ucid)?;
        Some(host_command(format!("/ban {} {ban_days}", conn.uname)))
    }

    /// Returns an `/unban` packet.
    pub fn unban(&self, uname: impl Into<String>) -> insim::Packet {
        host_command(format!("/unban {}", uname.into()))
    }

    /// Returns a `/spec` packet for the given UCID, or `None` if not found.
    pub fn spec(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        let conn = self.get(ucid)?;
        Some(host_command(format!("/spec {}", conn.uname)))
    }

    /// Returns a `/pitlane` packet for the given UCID, or `None` if not found.
    pub fn pitlane(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        let conn = self.get(ucid)?;
        Some(host_command(format!("/pitlane {}", conn.uname)))
    }

    /// Returns a `/p_clear` packet for the given UCID, or `None` if not found.
    pub fn clear_penalty(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        let conn = self.get(ucid)?;
        Some(host_command(format!("/p_clear {}", conn.uname)))
    }

    /// Returns the packets needed to set and display a Race Control Message.
    ///
    /// Pass [`ConnectionId::ALL`] to broadcast to all connections.
    /// Returns up to 2 packets; send them all.
    pub fn send_rcm(&self, message: &str, ucid: ConnectionId) -> Vec<insim::Packet> {
        let mut packets = vec![host_command(format!("/rcm {message}"))];
        if ucid == ConnectionId::ALL {
            packets.push(host_command("/rcm_all"));
        } else if let Some(conn) = self.get(ucid) {
            packets.push(host_command(format!("/rcm_ply {}", conn.uname)));
        }
        packets
    }

    /// Returns the packets needed to clear a Race Control Message.
    ///
    /// Pass [`ConnectionId::ALL`] to clear for everyone.
    pub fn clear_rcm(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        if ucid == ConnectionId::ALL {
            return Some(host_command("/rcc_all"));
        }
        let conn = self.get(ucid)?;
        Some(host_command(format!("/rcc_ply {}", conn.uname)))
    }

    /// Returns a penalty packet for the given UCID. Returns `None` if the
    /// UCID is not found or the penalty variant is not issueable.
    pub fn give_penalty(&self, ucid: ConnectionId, penalty: PenaltyInfo) -> Option<insim::Packet> {
        let conn = self.get(ucid)?;
        let cmd = match penalty {
            PenaltyInfo::Dt => format!("/p_dt {}", conn.uname),
            PenaltyInfo::Sg => format!("/p_sg {}", conn.uname),
            PenaltyInfo::Seconds30 => format!("/p_30 {}", conn.uname),
            PenaltyInfo::Seconds45 => format!("/p_45 {}", conn.uname),
            _ => return None,
        };
        Some(host_command(cmd))
    }

    /// Poll until the connection count satisfies `f`, or until `cancel` fires.
    /// Returns `Some(count)` on success or `None` if cancelled.
    pub async fn wait_for_connection_count<F: Fn(usize) -> bool>(
        &self,
        f: F,
        poll_interval: Duration,
        cancel: CancellationToken,
    ) -> Option<usize> {
        let mut interval = tokio::time::interval(poll_interval);
        loop {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => return None,
                _ = interval.tick() => {
                    let count = self.connection_count();
                    if f(count) {
                        return Some(count);
                    }
                }
            }
        }
    }

    /// Poll until the player count satisfies `f`, or until `cancel` fires.
    pub async fn wait_for_player_count<F: Fn(usize) -> bool>(
        &self,
        f: F,
        poll_interval: Duration,
        cancel: CancellationToken,
    ) -> Option<usize> {
        let mut interval = tokio::time::interval(poll_interval);
        loop {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => return None,
                _ = interval.tick() => {
                    let count = self.player_count();
                    if f(count) {
                        return Some(count);
                    }
                }
            }
        }
    }

    /// Apply one raw packet to the internal state maps without returning
    /// any events. Use when you only need queryable state.
    pub fn apply(&self, packet: &insim::Packet) {
        let _ = self.apply_events(packet);
    }

    /// Apply one raw packet and return the resulting state-change events.
    /// Also mutates internal state - call this instead of [`apply`](Self::apply)
    /// when you want to react to changes.
    pub fn apply_events(&self, packet: &insim::Packet) -> Vec<PresenceEvent> {
        match packet {
            insim::Packet::Ncn(ncn) => vec![PresenceEvent::Connected(self.apply_ncn(ncn))],
            insim::Packet::Cnl(cnl) => {
                let (info, players) = self.apply_cnl(cnl);
                let mut events: Vec<PresenceEvent> =
                    players.into_iter().map(PresenceEvent::PlayerLeft).collect();
                events.push(PresenceEvent::Disconnected {
                    ucid: cnl.ucid,
                    info,
                });
                events
            },
            insim::Packet::Nci(nci) => {
                if let Some(conn) = self.apply_nci(nci) {
                    vec![PresenceEvent::ConnectionDetails(conn)]
                } else {
                    vec![]
                }
            },
            insim::Packet::Slc(slc) => {
                if self.apply_slc(slc) {
                    vec![PresenceEvent::VehicleSelected {
                        ucid: slc.ucid,
                        vehicle: slc.cname,
                    }]
                } else {
                    vec![]
                }
            },
            insim::Packet::Cpr(cpr) => {
                if let Some((ucid, uname, new_pname)) = self.apply_cpr(cpr) {
                    vec![PresenceEvent::Renamed {
                        ucid,
                        uname,
                        new_pname,
                    }]
                } else {
                    vec![]
                }
            },
            insim::Packet::Npl(npl) => {
                if let Some(p) = self.apply_npl(npl) {
                    vec![PresenceEvent::PlayerJoined(p)]
                } else {
                    vec![]
                }
            },
            insim::Packet::Pll(pll) => {
                if let Some(p) = self.apply_pll(pll) {
                    vec![PresenceEvent::PlayerLeft(p)]
                } else {
                    vec![]
                }
            },
            insim::Packet::Toc(toc) => {
                if let Some((before, after)) = self.apply_toc(toc) {
                    vec![PresenceEvent::TakingOver { before, after }]
                } else {
                    vec![]
                }
            },
            insim::Packet::Pfl(pfl) => {
                self.apply_pfl(pfl);
                vec![]
            },
            insim::Packet::Pla(pla) => {
                self.apply_pla(pla);
                vec![]
            },
            insim::Packet::Plp(plp) => {
                let player = self
                    .inner
                    .read()
                    .expect("poison")
                    .players
                    .get(&plp.plid)
                    .cloned();
                if let Some(p) = player {
                    vec![PresenceEvent::PlayerTeleportedToPits(p)]
                } else {
                    vec![]
                }
            },
            insim::Packet::Tiny(tiny) => {
                self.apply_tiny_clr(tiny);
                vec![]
            },
            _ => vec![],
        }
    }

    fn apply_ncn(&self, ncn: &Ncn) -> ConnectionInfo {
        let info = ConnectionInfo {
            ucid: ncn.ucid,
            uname: ncn.uname.clone(),
            pname: ncn.pname.clone(),
            admin: ncn.admin,
            players: HashSet::new(),
            userid: None,
            ipaddress: None,
            selected_vehicle: None,
        };
        let mut guard = self.inner.write().expect("poison");
        let _ = guard
            .last_known_names
            .insert(ncn.uname.clone(), ncn.pname.clone());
        let _ = guard.connections.insert(info.ucid, info.clone());
        info
    }

    fn apply_cnl(&self, cnl: &Cnl) -> (Option<ConnectionInfo>, Vec<PlayerInfo>) {
        let mut guard = self.inner.write().expect("poison");
        let info = guard.connections.remove(&cnl.ucid);
        let mut left = Vec::new();
        if let Some(ref conn) = info {
            for plid in &conn.players {
                if let Some(p) = guard.players.remove(plid) {
                    left.push(p);
                }
            }
        }
        (info, left)
    }

    fn apply_nci(&self, nci: &Nci) -> Option<ConnectionInfo> {
        let mut guard = self.inner.write().expect("poison");
        if let Some(conn) = guard.connections.get_mut(&nci.ucid) {
            conn.userid = Some(nci.userid);
            conn.ipaddress = Some(nci.ipaddress);
            Some(conn.clone())
        } else {
            None
        }
    }

    fn apply_slc(&self, slc: &Slc) -> bool {
        let mut guard = self.inner.write().expect("poison");
        if let Some(conn) = guard.connections.get_mut(&slc.ucid) {
            conn.selected_vehicle = Some(slc.cname);
            true
        } else {
            false
        }
    }

    fn apply_cpr(&self, cpr: &Cpr) -> Option<(ConnectionId, String, String)> {
        let mut guard = self.inner.write().expect("poison");
        if let Some(conn) = guard.connections.get_mut(&cpr.ucid) {
            conn.pname = cpr.pname.clone();
            let uname = conn.uname.clone();
            let _ = guard
                .last_known_names
                .insert(uname.clone(), cpr.pname.clone());
            Some((cpr.ucid, uname, cpr.pname.clone()))
        } else {
            None
        }
    }

    fn apply_npl(&self, npl: &Npl) -> Option<PlayerInfo> {
        // A join request is signalled by `nump == 0`; the real join arrives
        // as a subsequent `Npl` with `nump` set.
        if npl.nump == 0 {
            return None;
        }
        let player = PlayerInfo {
            plid: npl.plid,
            ucid: npl.ucid,
            vehicle: npl.cname,
            ptype: npl.ptype,
            flags: npl.flags,
            in_pitlane: false,
            pname: npl.pname.clone(),
        };
        let mut guard = self.inner.write().expect("poison");
        let _ = guard.players.insert(npl.plid, player.clone());
        if let Some(conn) = guard.connections.get_mut(&npl.ucid) {
            let _ = conn.players.insert(npl.plid);
        }
        Some(player)
    }

    fn apply_pll(&self, pll: &Pll) -> Option<PlayerInfo> {
        let mut guard = self.inner.write().expect("poison");
        let player = guard.players.remove(&pll.plid);
        if let Some(ref p) = player
            && let Some(conn) = guard.connections.get_mut(&p.ucid)
        {
            let _ = conn.players.remove(&p.plid);
        }
        player
    }

    fn apply_toc(&self, toc: &Toc) -> Option<(PlayerInfo, PlayerInfo)> {
        let mut guard = self.inner.write().expect("poison");
        let pair = if let Some(player) = guard.players.get_mut(&toc.plid) {
            let before = player.clone();
            player.ucid = toc.newucid;
            let after = player.clone();
            Some((before, after))
        } else {
            None
        };
        if let Some(old) = guard.connections.get_mut(&toc.olducid) {
            old.players.retain(|&p| p != toc.plid);
        }
        if let Some(new) = guard.connections.get_mut(&toc.newucid) {
            let _ = new.players.insert(toc.plid);
        }
        pair
    }

    fn apply_pfl(&self, pfl: &Pfl) {
        let mut guard = self.inner.write().expect("poison");
        if let Some(player) = guard.players.get_mut(&pfl.plid) {
            player.flags = pfl.flags;
        }
    }

    fn apply_pla(&self, pla: &Pla) {
        let mut guard = self.inner.write().expect("poison");
        if let Some(player) = guard.players.get_mut(&pla.plid) {
            if pla.entered_pitlane() {
                player.in_pitlane = true;
            }
            if pla.exited_pitlane() {
                player.in_pitlane = false;
            }
        }
    }

    fn apply_tiny_clr(&self, tiny: &Tiny) {
        if !matches!(tiny.subt, TinyType::Clr) {
            return;
        }
        let mut guard = self.inner.write().expect("poison");
        guard.players.clear();
        for conn in guard.connections.values_mut() {
            conn.players.clear();
        }
    }
}
