//! [`Presence`] - tracks connections + players, emits lifecycle synthetic
//! events. Register via [`crate::App::install`]:
//!
//! ```ignore
//! app.install(Presence::new(sender.clone()))
//! ```
//!
//! The `presence_on_*` handlers below mutate the resource's state under its
//! lock, drop the guard, then emit a synthetic event the same dispatch cycle.
//! Downstream handlers see the emitted event in a *subsequent* cycle (per
//! the framework's emission semantics), where Presence's update has already
//! landed.

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
        Cnl, Cpr, Nci, Ncn, Npl, Pfl, Pla, Pll, PlayerFlags, PlayerType, Plp, Slc, Tiny, TinyType,
        Toc,
    },
};
use tokio_util::sync::CancellationToken;

use crate::{
    App, AppError, Installable,
    extract::{ExtractCx, FromContext, Packet, Sender},
    util::host_command,
};

// ---------------------------------------------------------------------------
// Presence - tracks connections + players, emits lifecycle events.
// ---------------------------------------------------------------------------

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

/// Synthetic event emitted when a connection joins.
#[derive(Debug, Clone)]
pub struct Connected(pub ConnectionInfo);

/// Synthetic event emitted when a connection leaves.
#[derive(Debug, Clone)]
pub struct Disconnected {
    /// The connection that left.
    pub ucid: ConnectionId,
    /// Last known info for the connection (cloned out of the live map).
    pub info: Option<ConnectionInfo>,
}

/// Synthetic event emitted when a connection changes their display name (`Cpr`).
#[derive(Debug, Clone)]
pub struct Renamed {
    /// Connection ID.
    pub ucid: ConnectionId,
    /// LFS.net username (stable - does not change on rename).
    pub uname: String,
    /// New display name.
    pub new_pname: String,
}

/// Synthetic event emitted when extra connection details arrive (`Nci` -
/// host-only, requires admin password).
#[derive(Debug, Clone)]
pub struct ConnectionDetails(pub ConnectionInfo);

/// Synthetic event emitted when a connection selects a vehicle in the
/// garage (`Slc`).
#[derive(Debug, Clone)]
pub struct VehicleSelected {
    /// Connection that selected the vehicle.
    pub ucid: ConnectionId,
    /// The selected vehicle.
    pub vehicle: Vehicle,
}

/// Synthetic event emitted when a player joins the track (`Npl`).
#[derive(Debug, Clone)]
pub struct PlayerJoined(pub PlayerInfo);

/// Synthetic event emitted when a player leaves the track (`Pll`).
#[derive(Debug, Clone)]
pub struct PlayerLeft(pub PlayerInfo);

/// Synthetic event emitted when a player's controlling connection changes
/// (driver swap via `Toc`).
#[derive(Debug, Clone)]
pub struct TakingOver {
    /// The player before the swap (old `ucid`).
    pub before: PlayerInfo,
    /// The player after the swap (new `ucid`).
    pub after: PlayerInfo,
}

/// Synthetic event emitted when a player tele-pits (Shift+P via `Plp`).
#[derive(Debug, Clone)]
pub struct PlayerTeleportedToPits(pub PlayerInfo);

#[derive(Default)]
struct PresenceInner {
    connections: HashMap<ConnectionId, ConnectionInfo>,
    players: HashMap<PlayerId, PlayerInfo>,
    /// Survives `Cnl`: maps LFS.net username → last seen display name.
    last_known_names: HashMap<String, String>,
}

/// Resource that tracks active connections and players. Pair with the
/// `presence_on_*` handlers (see [`Presence::install`]) to populate it from
/// wire packets and emit lifecycle synthetic events.
///
/// Internal state lives behind `Arc<RwLock<…>>`; clones of the handle are
/// cheap and all observe the same maps.
///
/// Admin commands (`kick` / `ban` / `unban` / `spec` / `pitlane`) send
/// `/command <uname>` host commands through the captured [`Sender`]. They
/// are fire-and-forget (matching `Sender::packet` semantics); there is no
/// confirmation from LFS.
///
/// ```ignore
/// let app = App::new().install(Presence::new(app.sender().clone()));
///
/// async fn handler(presence: Presence) -> Result<(), AppError> {
///     tracing::info!(
///         connections = presence.connection_count(),
///         players = presence.player_count(),
///         "current"
///     );
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct Presence {
    inner: Arc<RwLock<PresenceInner>>,
    sender: Sender,
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
    /// Create a new presence with empty state. `sender` is used to dispatch
    /// admin commands; obtain it from [`crate::App::sender`] before
    /// installing.
    pub fn new(sender: Sender) -> Self {
        Self {
            inner: Arc::new(RwLock::new(PresenceInner::default())),
            sender,
        }
    }

    /// Number of tracked connections. Alias for [`Presence::connection_count`].
    pub fn count(&self) -> usize {
        self.connection_count()
    }

    /// Number of tracked connections.
    pub fn connection_count(&self) -> usize {
        self.inner.read().expect("poison").connections.len()
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

    /// Batch variant of [`Presence::last_known_name`].
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

    /// Kick a connection. Fire-and-forget.
    pub fn kick(&self, ucid: ConnectionId) -> Result<(), AppError> {
        let Some(conn) = self.get(ucid) else { return Ok(()); };
        self.sender
            .packet(host_command(format!("/kick {}", conn.uname)))
    }

    /// Ban a connection. `ban_days` of 0 = 12 hours per LFS convention.
    pub fn ban(&self, ucid: ConnectionId, ban_days: u32) -> Result<(), AppError> {
        let Some(conn) = self.get(ucid) else { return Ok(()); };
        self.sender
            .packet(host_command(format!("/ban {} {ban_days}", conn.uname)))
    }

    /// Unban an LFS username.
    pub fn unban(&self, uname: impl Into<String>) -> Result<(), AppError> {
        let uname = uname.into();
        self.sender.packet(host_command(format!("/unban {uname}")))
    }

    /// Force a connection to spectate.
    pub fn spec(&self, ucid: ConnectionId) -> Result<(), AppError> {
        let Some(conn) = self.get(ucid) else { return Ok(()); };
        self.sender
            .packet(host_command(format!("/spec {}", conn.uname)))
    }

    /// Send a connection to the pit lane.
    pub fn pitlane(&self, ucid: ConnectionId) -> Result<(), AppError> {
        let Some(conn) = self.get(ucid) else { return Ok(()); };
        self.sender
            .packet(host_command(format!("/pitlane {}", conn.uname)))
    }

    /// Poll until the connection count satisfies `f`, or until `cancel`
    /// fires. Returns `Some(count)` on success or `None` if cancelled.
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
}

/// [`Presence`] is its own extractor: register via `App::install(presence)`
/// and any handler can take it by value.
impl FromContext for Presence {
    fn from_context(cx: &ExtractCx<'_>) -> Option<Self> {
        cx.resources.get::<Presence>()
    }
}

impl Installable for Presence {
    fn install(self, app: App) -> App {
        app.resource(self)
            .handler(presence_on_ncn)
            .handler(presence_on_cnl)
            .handler(presence_on_nci)
            .handler(presence_on_slc)
            .handler(presence_on_cpr)
            .handler(presence_on_npl)
            .handler(presence_on_pll)
            .handler(presence_on_toc)
            .handler(presence_on_pfl)
            .handler(presence_on_pla)
            .handler(presence_on_plp)
            .handler(presence_on_tiny_clr)
    }
}

// ---------------------------------------------------------------------------
// Presence: handler functions
//
// Each handler is a regular extractor-driven async fn. They mutate the
// Presence resource under its lock, drop the guard, then emit any synthetic
// event - the same pattern the old Extension::on_event used, just split per
// packet variant.
// ---------------------------------------------------------------------------

async fn presence_on_ncn(
    Packet(ncn): Packet<Ncn>,
    presence: Presence,
    sender: Sender,
) -> Result<(), AppError> {
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
    {
        let mut guard = presence.inner.write().expect("poison");
        let _ = guard
            .last_known_names
            .insert(ncn.uname.clone(), ncn.pname.clone());
        let _ = guard.connections.insert(info.ucid, info.clone());
    }
    let _ = sender.event(Connected(info));
    Ok(())
}

async fn presence_on_cnl(
    Packet(cnl): Packet<Cnl>,
    presence: Presence,
    sender: Sender,
) -> Result<(), AppError> {
    let (info, departed_players) = {
        let mut guard = presence.inner.write().expect("poison");
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
    };
    for player in departed_players {
        let _ = sender.event(PlayerLeft(player));
    }
    let _ = sender.event(Disconnected {
        ucid: cnl.ucid,
        info,
    });
    Ok(())
}

async fn presence_on_nci(
    Packet(nci): Packet<Nci>,
    presence: Presence,
    sender: Sender,
) -> Result<(), AppError> {
    let updated = {
        let mut guard = presence.inner.write().expect("poison");
        if let Some(conn) = guard.connections.get_mut(&nci.ucid) {
            conn.userid = Some(nci.userid);
            conn.ipaddress = Some(nci.ipaddress);
            Some(conn.clone())
        } else {
            None
        }
    };
    if let Some(conn) = updated {
        let _ = sender.event(ConnectionDetails(conn));
    }
    Ok(())
}

async fn presence_on_slc(
    Packet(slc): Packet<Slc>,
    presence: Presence,
    sender: Sender,
) -> Result<(), AppError> {
    let updated = {
        let mut guard = presence.inner.write().expect("poison");
        if let Some(conn) = guard.connections.get_mut(&slc.ucid) {
            conn.selected_vehicle = Some(slc.cname);
            true
        } else {
            false
        }
    };
    if updated {
        let _ = sender.event(VehicleSelected {
            ucid: slc.ucid,
            vehicle: slc.cname,
        });
    }
    Ok(())
}

async fn presence_on_cpr(
    Packet(cpr): Packet<Cpr>,
    presence: Presence,
    sender: Sender,
) -> Result<(), AppError> {
    let event = {
        let mut guard = presence.inner.write().expect("poison");
        if let Some(conn) = guard.connections.get_mut(&cpr.ucid) {
            conn.pname = cpr.pname.clone();
            let uname = conn.uname.clone();
            let _ = guard
                .last_known_names
                .insert(uname.clone(), cpr.pname.clone());
            Some(Renamed {
                ucid: cpr.ucid,
                uname,
                new_pname: cpr.pname.clone(),
            })
        } else {
            None
        }
    };
    if let Some(e) = event {
        let _ = sender.event(e);
    }
    Ok(())
}

async fn presence_on_npl(
    Packet(npl): Packet<Npl>,
    presence: Presence,
    sender: Sender,
) -> Result<(), AppError> {
    // A join request is signalled by `nump == 0`; the real join arrives
    // as a subsequent `Npl` with `nump` set.
    if npl.nump == 0 {
        return Ok(());
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
    {
        let mut guard = presence.inner.write().expect("poison");
        let _ = guard.players.insert(npl.plid, player.clone());
        if let Some(conn) = guard.connections.get_mut(&npl.ucid) {
            let _ = conn.players.insert(npl.plid);
        }
    }
    let _ = sender.event(PlayerJoined(player));
    Ok(())
}

async fn presence_on_pll(
    Packet(pll): Packet<Pll>,
    presence: Presence,
    sender: Sender,
) -> Result<(), AppError> {
    let player = {
        let mut guard = presence.inner.write().expect("poison");
        let player = guard.players.remove(&pll.plid);
        if let Some(ref p) = player
            && let Some(conn) = guard.connections.get_mut(&p.ucid)
        {
            let _ = conn.players.remove(&p.plid);
        }
        player
    };
    if let Some(p) = player {
        let _ = sender.event(PlayerLeft(p));
    }
    Ok(())
}

async fn presence_on_toc(
    Packet(toc): Packet<Toc>,
    presence: Presence,
    sender: Sender,
) -> Result<(), AppError> {
    let pair = {
        let mut guard = presence.inner.write().expect("poison");
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
    };
    if let Some((before, after)) = pair {
        let _ = sender.event(TakingOver { before, after });
    }
    Ok(())
}

async fn presence_on_pfl(Packet(pfl): Packet<Pfl>, presence: Presence) -> Result<(), AppError> {
    let mut guard = presence.inner.write().expect("poison");
    if let Some(player) = guard.players.get_mut(&pfl.plid) {
        player.flags = pfl.flags;
    }
    Ok(())
}

async fn presence_on_pla(Packet(pla): Packet<Pla>, presence: Presence) -> Result<(), AppError> {
    let mut guard = presence.inner.write().expect("poison");
    if let Some(player) = guard.players.get_mut(&pla.plid) {
        if pla.entered_pitlane() {
            player.in_pitlane = true;
        }
        if pla.exited_pitlane() {
            player.in_pitlane = false;
        }
    }
    Ok(())
}

async fn presence_on_plp(
    Packet(plp): Packet<Plp>,
    presence: Presence,
    sender: Sender,
) -> Result<(), AppError> {
    let player = presence
        .inner
        .read()
        .expect("poison")
        .players
        .get(&plp.plid)
        .cloned();
    if let Some(p) = player {
        let _ = sender.event(PlayerTeleportedToPits(p));
    }
    Ok(())
}

async fn presence_on_tiny_clr(
    Packet(tiny): Packet<Tiny>,
    presence: Presence,
) -> Result<(), AppError> {
    if !matches!(tiny.subt, TinyType::Clr) {
        return Ok(());
    }
    let mut guard = presence.inner.write().expect("poison");
    guard.players.clear();
    for conn in guard.connections.values_mut() {
        conn.players.clear();
    }
    Ok(())
}

