//! [`Presence`] tracks connections and players from a bare `insim` packet
//! stream. Feed packets in, query state out.
//!
//! Admin commands return [`insim::Packet`] values for the caller to send;
//! multi-packet commands return [`Vec<insim::Packet>`].
//!
//! ```ignore
//! let presence = Presence::new();
//!
//! while let Some(packet) = conn.next().await {
//!     for event in presence.apply_packet(&packet) {
//!         match event {
//!             PresenceEvent::Connected(info) => println!("{} joined", info.pname),
//!             _ => {}
//!         }
//!     }
//! }
//! ```

mod commands;

use std::{collections::HashMap, net::Ipv4Addr, sync::Arc, time::Duration};

use insim::{
    core::vehicle::Vehicle,
    identifiers::{ConnectionId, PlayerId},
    insim::{Cnl, Cpr, Nci, Ncn, Npl, Pfl, Pla, Pll, Slc, Tiny, TinyType, Toc},
};
use parking_lot::RwLock;
use tokio_util::sync::CancellationToken;

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
    /// LFS.net user ID. Only populated when the host has an admin password
    /// set and an `Nci` packet has been received for this connection.
    pub userid: Option<u32>,
    /// Originating IP address. Only populated on hosts with an admin
    /// password (via `Nci`).
    pub ipaddress: Option<Ipv4Addr>,
    /// Most recently selected vehicle in the garage (from `Slc`).
    pub selected_vehicle: Option<Vehicle>,
}

// XXX: `MultiIndexMap` generates a map struct and associated methods that cannot
// carry doc comments, so `missing_docs` is suppressed for this module only.
#[allow(missing_docs)]
mod player_info {
    use insim::{
        core::vehicle::Vehicle,
        identifiers::{ConnectionId, PlayerId},
        insim::{PlayerFlags, PlayerType},
    };
    use multi_index_map::MultiIndexMap;

    /// Per-player record stored by [`super::Presence`].
    #[derive(MultiIndexMap, Debug, Clone)]
    pub struct PlayerInfo {
        /// Player ID.
        #[multi_index(hashed_unique)]
        pub plid: PlayerId,
        /// Owning connection ID.
        #[multi_index(hashed_non_unique)]
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
}

use player_info::MultiIndexPlayerInfoMap;
pub use player_info::PlayerInfo;

/// State-change events produced by [`Presence::apply_packet`].
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
    players: MultiIndexPlayerInfoMap,
    /// Survives `Cnl`: maps LFS.net username → last seen display name.
    last_known_names: HashMap<String, String>,
}

/// Tracks active connections and players from a stream of `insim` packets.
///
/// State lives behind `Arc<RwLock<…>>`; clones are cheap and share the same
/// maps.
///
/// Feed packets with [`apply_packet`](Self::apply_packet) to update state and
/// collect change events. Admin commands return packets for the caller to send.
// TODO: Merge directly into World
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
    /// Tiny requests to send once on connect to sync the current connection
    /// and player lists. LFS does not send these automatically on connect.
    pub const STARTUP_REQUESTS: &[TinyType] = &[TinyType::Ncn, TinyType::Npl];

    /// Create a new presence tracker with empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of tracked connections.
    pub fn connection_count(&self) -> usize {
        self.inner.read().connections.len()
    }

    /// Alias for [`connection_count`](Self::connection_count).
    pub fn count(&self) -> usize {
        self.connection_count()
    }

    /// Number of tracked players.
    pub fn player_count(&self) -> usize {
        self.inner.read().players.len()
    }

    /// Look up one connection by UCID.
    pub fn get(&self, ucid: ConnectionId) -> Option<ConnectionInfo> {
        self.inner.read().connections.get(&ucid).cloned()
    }

    /// Snapshot of all tracked connections.
    pub fn connections(&self) -> Vec<ConnectionInfo> {
        self.inner.read().connections.values().cloned().collect()
    }

    /// Look up one player by `PlayerId`.
    pub fn player(&self, plid: PlayerId) -> Option<PlayerInfo> {
        self.inner.read().players.get_by_plid(&plid).cloned()
    }

    /// Snapshot of all tracked players.
    pub fn players(&self) -> Vec<PlayerInfo> {
        self.inner
            .read()
            .players
            .iter()
            .map(|(_, p)| p.clone())
            .collect()
    }

    /// All players currently owned by a given connection.
    pub fn players_by_connection(&self, ucid: ConnectionId) -> Vec<PlayerInfo> {
        self.inner
            .read()
            .players
            .get_by_ucid(&ucid)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Look up the connection that currently controls a given player.
    pub fn connection_by_player(&self, plid: PlayerId) -> Option<ConnectionInfo> {
        let guard = self.inner.read();
        let player = guard.players.get_by_plid(&plid)?;
        guard.connections.get(&player.ucid).cloned()
    }

    /// Last known display name for an LFS.net username. Survives disconnect.
    pub fn last_known_name(&self, uname: &str) -> Option<String> {
        self.inner.read().last_known_names.get(uname).cloned()
    }

    /// Batch variant of [`last_known_name`](Self::last_known_name).
    pub fn last_known_names<I, S>(&self, unames: I) -> HashMap<String, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let guard = self.inner.read();
        unames
            .into_iter()
            .filter_map(|u| {
                let u = u.into();
                guard.last_known_names.get(&u).map(|p| (u, p.clone()))
            })
            .collect()
    }

    /// Poll until the connection count satisfies `f`, or until `cancel` fires.
    /// Returns `Some(count)` on success or `None` if cancelled.
    pub async fn wait_for_connection_count<F: Fn(usize) -> bool>(
        &self,
        f: F,
        poll_interval: Duration,
        cancel: CancellationToken,
    ) -> Option<usize> {
        self.wait_for_count(Self::connection_count, f, poll_interval, cancel)
            .await
    }

    /// Poll until the player count satisfies `f`, or until `cancel` fires.
    pub async fn wait_for_player_count<F: Fn(usize) -> bool>(
        &self,
        f: F,
        poll_interval: Duration,
        cancel: CancellationToken,
    ) -> Option<usize> {
        self.wait_for_count(Self::player_count, f, poll_interval, cancel)
            .await
    }

    async fn wait_for_count<F: Fn(usize) -> bool>(
        &self,
        count_fn: fn(&Self) -> usize,
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
                    let count = count_fn(self);
                    if f(count) {
                        return Some(count);
                    }
                }
            }
        }
    }

    /// Apply one raw packet, update internal state, and return any state-change events.
    pub fn apply_packet(&self, packet: &insim::Packet) -> Vec<PresenceEvent> {
        match packet {
            insim::Packet::Ncn(ncn) => vec![PresenceEvent::Connected(self.apply_ncn(ncn))],
            insim::Packet::Cnl(cnl) => {
                let (info, players) = self.apply_cnl(cnl);
                players
                    .into_iter()
                    .map(PresenceEvent::PlayerLeft)
                    .chain(std::iter::once(PresenceEvent::Disconnected {
                        ucid: cnl.ucid,
                        info,
                    }))
                    .collect()
            },
            insim::Packet::Nci(nci) => self
                .apply_nci(nci)
                .map(PresenceEvent::ConnectionDetails)
                .into_iter()
                .collect(),
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
            insim::Packet::Cpr(cpr) => self
                .apply_cpr(cpr)
                .map(|(ucid, uname, new_pname)| PresenceEvent::Renamed {
                    ucid,
                    uname,
                    new_pname,
                })
                .into_iter()
                .collect(),
            insim::Packet::Npl(npl) => self
                .apply_npl(npl)
                .map(PresenceEvent::PlayerJoined)
                .into_iter()
                .collect(),
            insim::Packet::Pll(pll) => self
                .apply_pll(pll)
                .map(PresenceEvent::PlayerLeft)
                .into_iter()
                .collect(),
            insim::Packet::Toc(toc) => self
                .apply_toc(toc)
                .map(|(before, after)| PresenceEvent::TakingOver { before, after })
                .into_iter()
                .collect(),
            insim::Packet::Pfl(pfl) => {
                self.apply_pfl(pfl);
                vec![]
            },
            insim::Packet::Pla(pla) => {
                self.apply_pla(pla);
                vec![]
            },
            insim::Packet::Plp(plp) => {
                let player = self.inner.read().players.get_by_plid(&plp.plid).cloned();
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
            userid: None,
            ipaddress: None,
            selected_vehicle: None,
        };
        let mut guard = self.inner.write();
        let _ = guard
            .last_known_names
            .insert(ncn.uname.clone(), ncn.pname.clone());
        let _ = guard.connections.insert(info.ucid, info.clone());
        info
    }

    fn apply_cnl(&self, cnl: &Cnl) -> (Option<ConnectionInfo>, Vec<PlayerInfo>) {
        let mut guard = self.inner.write();
        let info = guard.connections.remove(&cnl.ucid);
        let left = guard.players.remove_by_ucid(&cnl.ucid);
        (info, left)
    }

    fn apply_nci(&self, nci: &Nci) -> Option<ConnectionInfo> {
        let mut guard = self.inner.write();
        let conn = guard.connections.get_mut(&nci.ucid)?;
        conn.userid = Some(nci.userid);
        conn.ipaddress = Some(nci.ipaddress);
        Some(conn.clone())
    }

    fn apply_slc(&self, slc: &Slc) -> bool {
        let mut guard = self.inner.write();
        if let Some(conn) = guard.connections.get_mut(&slc.ucid) {
            conn.selected_vehicle = Some(slc.cname);
            true
        } else {
            false
        }
    }

    fn apply_cpr(&self, cpr: &Cpr) -> Option<(ConnectionId, String, String)> {
        let mut guard = self.inner.write();
        let conn = guard.connections.get_mut(&cpr.ucid)?;
        conn.pname = cpr.pname.clone();
        let uname = conn.uname.clone();
        let _ = guard
            .last_known_names
            .insert(uname.clone(), cpr.pname.clone());
        Some((cpr.ucid, uname, cpr.pname.clone()))
    }

    fn apply_npl(&self, npl: &Npl) -> Option<PlayerInfo> {
        let player = PlayerInfo {
            plid: npl.plid,
            ucid: npl.ucid,
            vehicle: npl.cname,
            ptype: npl.ptype,
            flags: npl.flags,
            in_pitlane: false,
            pname: npl.pname.clone(),
        };
        let _ = self.inner.write().players.insert(player.clone());
        // Only emit PlayerJoined once the join is confirmed (nump > 0).
        if npl.nump == 0 { None } else { Some(player) }
    }

    fn apply_pll(&self, pll: &Pll) -> Option<PlayerInfo> {
        self.inner.write().players.remove_by_plid(&pll.plid)
    }

    fn apply_toc(&self, toc: &Toc) -> Option<(PlayerInfo, PlayerInfo)> {
        let mut guard = self.inner.write();
        let before = guard.players.get_by_plid(&toc.plid)?.clone();
        let after = guard
            .players
            .modify_by_plid(&toc.plid, |p| p.ucid = toc.newucid)?
            .clone();
        Some((before, after))
    }

    fn apply_pfl(&self, pfl: &Pfl) {
        let _ = self
            .inner
            .write()
            .players
            .modify_by_plid(&pfl.plid, |p| p.flags = pfl.flags);
    }

    fn apply_pla(&self, pla: &Pla) {
        let _ = self.inner.write().players.modify_by_plid(&pla.plid, |p| {
            if pla.entered_pitlane() {
                p.in_pitlane = true;
            }
            if pla.exited_pitlane() {
                p.in_pitlane = false;
            }
        });
    }

    fn apply_tiny_clr(&self, tiny: &Tiny) {
        if !matches!(tiny.subt, TinyType::Clr) {
            return;
        }
        self.inner.write().players.clear();
    }
}
