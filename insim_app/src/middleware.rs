//! Extension trait, runtime context, and the bundled `Presence` /
//! `ChatParser<C>` helpers.
//!
//! An [`Extension<S>`] is a registered value that (a) handlers can pull out
//! via [`FromContext`] and (b) optionally observes every [`Dispatch`] through
//! the default-overridable `on_event` hook. Pure data extensions accept the
//! default no-op; ones that need to react (parse chat, track presence) override.
//! Either way, registration is a single `App::extension(value)` call.

use std::{
    any::Any,
    collections::{HashMap, HashSet},
    future::Future,
    marker::PhantomData,
    net::Ipv4Addr,
    str::FromStr,
    sync::{Arc, RwLock},
    time::Duration,
};

use futures::future::BoxFuture;
use insim::{
    core::vehicle::Vehicle,
    identifiers::{ConnectionId, PlayerId},
    insim::{MsoUserType, Ncn, PlayerFlags, PlayerType, TinyType},
};
use tokio_util::sync::CancellationToken;

use crate::{
    AppError,
    event::Dispatch,
    extensions::Extensions,
    extract::{ExtractCx, FromContext, Sender},
    util::host_command,
};

/// Context handed to an extension's `on_event`.
#[derive(Debug)]
pub struct EventCx<'a, S> {
    /// The current dispatch.
    pub dispatch: &'a Dispatch,
    /// Shared app state (read-only in the PoC).
    pub state: &'a S,
    /// Back-channel - same surface handlers see. Use `cx.sender.packet(...)` to
    /// send a wire packet, `cx.sender.event(...)` to inject a synthetic event
    /// into a subsequent dispatch cycle.
    pub sender: &'a Sender,
    /// Extension registry; same instance handlers see.
    pub extensions: &'a Extensions,
    /// Cooperative-shutdown token.
    pub cancel: &'a CancellationToken,
}

impl<'a, S> EventCx<'a, S> {
    /// Request graceful shutdown of the runtime.
    pub fn shutdown(&self) {
        self.cancel.cancel();
    }

    /// Whether shutdown has been requested.
    pub fn is_shutdown(&self) -> bool {
        self.cancel.is_cancelled()
    }
}

/// A value registered with the [`App`](crate::App). Combines two roles:
///
/// 1. **Extractor source** - every registered extension lives in the
///    [`Extensions`] registry by its `TypeId`, so handlers can pull it out via
///    [`FromContext`] (the extension author implements [`FromContext`] on the
///    type itself).
/// 2. **Optional event observer** - the default no-op `on_event` is overridden
///    by extensions that need to react to wire packets or synthetic events.
///
/// The trait takes `&self` (not `&mut self`) so the runtime can hold a single
/// `Arc<E>` shared between the registry and the dispatch chain - no `Clone`
/// bound on `E`, no per-registration clone. Stateful extensions manage their
/// own mutability internally (`Arc<RwLock<…>>` is the usual pattern).
///
/// Like [`crate::Handler`], the trait method is declared with `-> impl Future
/// + Send` so the runtime can require `Send` at the [`ErasedExtension`]
/// boundary; impls can still use `async fn` syntax.
pub trait Extension<S>: Send + Sync + 'static {
    /// Called for every dispatch (wire packets and synthetic events). Runs
    /// sequentially, in registration order, *before* handlers. Default is a
    /// no-op so pure data extensions don't have to write anything.
    fn on_event<'a>(&'a self, _cx: &'a mut EventCx<'_, S>) -> impl Future<Output = ()> + Send + 'a {
        async {}
    }
}

/// Object-safe shim so heterogeneous extensions can sit behind one Arc.
pub(crate) trait ErasedExtension<S>: Send + Sync {
    fn on_event<'a>(&'a self, cx: &'a mut EventCx<'_, S>) -> BoxFuture<'a, ()>;
}

impl<S, E> ErasedExtension<S> for E
where
    E: Extension<S>,
{
    fn on_event<'a>(&'a self, cx: &'a mut EventCx<'_, S>) -> BoxFuture<'a, ()> {
        Box::pin(<Self as Extension<S>>::on_event(self, cx))
    }
}

// ---------------------------------------------------------------------------
// Presence - tracks connections, emits Connected/Disconnected, and acts as
// its own extractor.
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

/// Synthetic event emitted when a connection changes their display name
/// (`Cpr`).
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

/// Synthetic event emitted when a player tele-pits (Shift+P via `Plp`). The
/// player is still in the race but repositioned in the pit lane.
#[derive(Debug, Clone)]
pub struct PlayerTeleportedToPits(pub PlayerInfo);

#[derive(Default)]
struct PresenceInner {
    connections: HashMap<ConnectionId, ConnectionInfo>,
    players: HashMap<PlayerId, PlayerInfo>,
    /// Survives `Cnl`: maps LFS.net username → last seen display name.
    last_known_names: HashMap<String, String>,
}

/// Extension that tracks active connections and players, emits synthetic
/// lifecycle events, and exposes admin commands.
///
/// Registers with `App::extension`; queryable from handlers via
/// `FromContext`. Internal state lives behind `Arc<RwLock<…>>`; clones of
/// the handle are cheap and all observe the same maps.
///
/// Admin commands (`kick` / `ban` / `unban` / `spec` / `pitlane`) send
/// `/command <uname>` host commands through the captured [`Sender`]. They
/// are fire-and-forget (matching `Sender::packet` semantics); there is no
/// confirmation from LFS.
///
/// ```ignore
/// let app = App::new()
///     .with_state(MyState { ... })
///     .extension(Presence::new(app.sender().clone()));
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
    /// Create a new presence extension with empty state. `sender` is used to
    /// dispatch admin commands; obtain it from [`crate::App::sender`] before
    /// registering the presence via `.extension(...)`.
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

    /// Number of tracked players (track participants - subset of connections;
    /// AI counts).
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

    /// Last known display name for an LFS.net username. Survives disconnect -
    /// useful for attributing async work (DB writes, etc.) to a player who
    /// may have already left.
    pub fn last_known_name(&self, uname: &str) -> Option<String> {
        self.inner
            .read()
            .expect("poison")
            .last_known_names
            .get(uname)
            .cloned()
    }

    /// Batch variant of [`Presence::last_known_name`]. Only unames with a
    /// known display name appear in the returned map.
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

    /// Kick a connection. Fire-and-forget; succeeds-by-default unless the
    /// runtime back-channel is closed.
    pub fn kick(&self, ucid: ConnectionId) -> Result<(), AppError> {
        let Some(conn) = self.get(ucid) else {
            return Ok(());
        };
        self.sender
            .packet(host_command(format!("/kick {}", conn.uname)))
    }

    /// Ban a connection. `ban_days` of 0 = 12 hours per LFS convention.
    pub fn ban(&self, ucid: ConnectionId, ban_days: u32) -> Result<(), AppError> {
        let Some(conn) = self.get(ucid) else {
            return Ok(());
        };
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
        let Some(conn) = self.get(ucid) else {
            return Ok(());
        };
        self.sender
            .packet(host_command(format!("/spec {}", conn.uname)))
    }

    /// Send a connection to the pit lane.
    pub fn pitlane(&self, ucid: ConnectionId) -> Result<(), AppError> {
        let Some(conn) = self.get(ucid) else {
            return Ok(());
        };
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
    /// Returns `Some(count)` on success or `None` if cancelled.
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

impl<S: Send + Sync + 'static> Extension<S> for Presence {
    async fn on_event(&self, cx: &mut EventCx<'_, S>) {
        let Dispatch::Packet(packet) = cx.dispatch else {
            return;
        };
        match packet {
            insim::Packet::Ncn(ncn) => self.handle_ncn(cx, ncn),
            insim::Packet::Cnl(cnl) => self.handle_cnl(cx, cnl),
            insim::Packet::Nci(nci) => self.handle_nci(cx, nci),
            insim::Packet::Slc(slc) => self.handle_slc(cx, slc),
            insim::Packet::Cpr(cpr) => self.handle_cpr(cx, cpr),
            insim::Packet::Npl(npl) => self.handle_npl(cx, npl),
            insim::Packet::Pll(pll) => self.handle_pll(cx, pll),
            insim::Packet::Toc(toc) => self.handle_toc(cx, toc),
            insim::Packet::Pfl(pfl) => self.handle_pfl(cx, pfl),
            insim::Packet::Pla(pla) => self.handle_pla(cx, pla),
            insim::Packet::Plp(plp) => self.handle_plp(cx, plp),
            insim::Packet::Tiny(tiny) if matches!(tiny.subt, TinyType::Clr) => {
                let mut guard = self.inner.write().expect("poison");
                guard.players.clear();
                for conn in guard.connections.values_mut() {
                    conn.players.clear();
                }
            },
            _ => {},
        }
    }
}

// Private packet-by-packet handlers. Each clones what it needs out from
// under the lock, then drops the guard before emitting the synthetic event
// (so handlers downstream can re-enter the lock if they want).
impl Presence {
    fn handle_ncn<S>(&self, cx: &EventCx<'_, S>, ncn: &Ncn) {
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
            let mut guard = self.inner.write().expect("poison");
            let _ = guard
                .last_known_names
                .insert(ncn.uname.clone(), ncn.pname.clone());
            let _ = guard.connections.insert(info.ucid, info.clone());
        }
        let _ = cx.sender.event(Connected(info));
    }

    fn handle_cnl<S>(&self, cx: &EventCx<'_, S>, cnl: &insim::insim::Cnl) {
        let (info, departed_players) = {
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
        };
        for player in departed_players {
            let _ = cx.sender.event(PlayerLeft(player));
        }
        let _ = cx.sender.event(Disconnected {
            ucid: cnl.ucid,
            info,
        });
    }

    fn handle_nci<S>(&self, cx: &EventCx<'_, S>, nci: &insim::insim::Nci) {
        let updated = {
            let mut guard = self.inner.write().expect("poison");
            if let Some(conn) = guard.connections.get_mut(&nci.ucid) {
                conn.userid = Some(nci.userid);
                conn.ipaddress = Some(nci.ipaddress);
                Some(conn.clone())
            } else {
                None
            }
        };
        if let Some(conn) = updated {
            let _ = cx.sender.event(ConnectionDetails(conn));
        }
    }

    fn handle_slc<S>(&self, cx: &EventCx<'_, S>, slc: &insim::insim::Slc) {
        let updated = {
            let mut guard = self.inner.write().expect("poison");
            if let Some(conn) = guard.connections.get_mut(&slc.ucid) {
                conn.selected_vehicle = Some(slc.cname);
                true
            } else {
                false
            }
        };
        if updated {
            let _ = cx.sender.event(VehicleSelected {
                ucid: slc.ucid,
                vehicle: slc.cname,
            });
        }
    }

    fn handle_cpr<S>(&self, cx: &EventCx<'_, S>, cpr: &insim::insim::Cpr) {
        let event = {
            let mut guard = self.inner.write().expect("poison");
            if let Some(conn) = guard.connections.get_mut(&cpr.ucid) {
                conn.pname = cpr.pname.clone();
                let uname = conn.uname.clone();
                let _ = guard.last_known_names.insert(uname.clone(), cpr.pname.clone());
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
            let _ = cx.sender.event(e);
        }
    }

    fn handle_npl<S>(&self, cx: &EventCx<'_, S>, npl: &insim::insim::Npl) {
        // A join request is signalled by `nump == 0`; the real join arrives
        // as a subsequent `Npl` with `nump` set.
        if npl.nump == 0 {
            return;
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
            let mut guard = self.inner.write().expect("poison");
            let _ = guard.players.insert(npl.plid, player.clone());
            if let Some(conn) = guard.connections.get_mut(&npl.ucid) {
                let _ = conn.players.insert(npl.plid);
            }
        }
        let _ = cx.sender.event(PlayerJoined(player));
    }

    fn handle_pll<S>(&self, cx: &EventCx<'_, S>, pll: &insim::insim::Pll) {
        let player = {
            let mut guard = self.inner.write().expect("poison");
            let player = guard.players.remove(&pll.plid);
            if let Some(ref p) = player
                && let Some(conn) = guard.connections.get_mut(&p.ucid)
            {
                let _ = conn.players.remove(&p.plid);
            }
            player
        };
        if let Some(p) = player {
            let _ = cx.sender.event(PlayerLeft(p));
        }
    }

    fn handle_toc<S>(&self, cx: &EventCx<'_, S>, toc: &insim::insim::Toc) {
        let pair = {
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
        };
        if let Some((before, after)) = pair {
            let _ = cx.sender.event(TakingOver { before, after });
        }
    }

    fn handle_pfl<S>(&self, _cx: &EventCx<'_, S>, pfl: &insim::insim::Pfl) {
        let mut guard = self.inner.write().expect("poison");
        if let Some(player) = guard.players.get_mut(&pfl.plid) {
            player.flags = pfl.flags;
        }
    }

    fn handle_pla<S>(&self, _cx: &EventCx<'_, S>, pla: &insim::insim::Pla) {
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

    fn handle_plp<S>(&self, cx: &EventCx<'_, S>, plp: &insim::insim::Plp) {
        let player = self
            .inner
            .read()
            .expect("poison")
            .players
            .get(&plp.plid)
            .cloned();
        if let Some(p) = player {
            let _ = cx.sender.event(PlayerTeleportedToPits(p));
        }
    }
}

/// [`Presence`] is its own extractor: register via [`crate::App::extension`]
/// and any handler can take it by value.
impl<S: Send + Sync + 'static> FromContext<S> for Presence {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        cx.extensions.get::<Presence>()
    }
}

// ---------------------------------------------------------------------------
// ChatParser<C> - typed Mso → C parser. Parses once, dispatches via Event<C>.
// ---------------------------------------------------------------------------

/// Extension that parses every `Mso` body into a typed value `C` via
/// [`FromStr`] and emits the parsed value as a synthetic event.
///
/// **The point of using this over a per-handler parse extractor is that the
/// parse runs once per `Mso` packet** regardless of how many `Event<C>`
/// handlers are registered - they all see the same `Arc`-wrapped value.
///
/// Pair with `Event<C>` handlers and the existing
/// `insim_extras::chat::Parse` derive (via a small `FromStr` bridge - see
/// docs on the example crate). On parse failure (wrong prefix, unknown
/// command, malformed args) no event is emitted.
pub struct ChatParser<C> {
    _phantom: PhantomData<fn() -> C>,
}

impl<C> ChatParser<C> {
    /// Create a new typed chat parser extension.
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<C> Default for ChatParser<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> std::fmt::Debug for ChatParser<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatParser").finish_non_exhaustive()
    }
}

impl<S, C> Extension<S> for ChatParser<C>
where
    C: FromStr + Any + Send + Sync + 'static,
    S: Send + Sync + 'static,
{
    async fn on_event(&self, cx: &mut EventCx<'_, S>) {
        let Dispatch::Packet(insim::Packet::Mso(mso)) = cx.dispatch else {
            return;
        };
        if !matches!(mso.usertype, MsoUserType::User | MsoUserType::Prefix) {
            return;
        }
        if let Ok(c) = mso.msg_from_textstart().trim().parse::<C>() {
            let _ = cx.sender.event(c);
        }
    }
}
