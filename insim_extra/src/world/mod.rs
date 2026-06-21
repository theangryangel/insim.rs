//! [`World`] is the unified state mirror combining connection/player tracking,
//! game-state mirroring, and race tracking in a single structure.
//!
//! Feed packets with [`apply_packet`](World::apply_packet) to update state and
//! collect change events. Query connection/player/game state directly on the
//! `World` value, and issue admin commands that return [`insim::Packet`] values
//! for the caller to send.
//!
//! ```ignore
//! let world = World::new();
//!
//! while let Some(packet) = conn.next().await {
//!     for event in world.apply_packet(&packet) {
//!         match event {
//!             WorldEvent::Connected(Connected(info)) => { /* ... */ }
//!             WorldEvent::SessionStarted(SessionStarted { kind }) => { /* ... */ }
//!             WorldEvent::Race(re) => { /* ... */ }
//!             _ => {}
//!         }
//!     }
//! }
//! ```

use std::{collections::HashMap, sync::Arc, time::Duration};

use insim::{
    core::{track::Track, vehicle::Vehicle, wind::Wind},
    identifiers::{ConnectionId, PlayerId},
    insim::{
        Axi, Cnl, Cpr, Ism, Nci, Ncn, Npl, Pfl, Pla, PlcAllowedCarsSet, Pll, RaceFlags,
        RaceInProgress, RaceLaps, Rst, Slc, SmallType, Sta, StaFlags, Tiny, TinyType, Toc, Ver,
    },
};
use parking_lot::RwLock;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

mod commands;
mod connection;
mod event;
mod game;
mod race;

use connection::MultiIndexPlayerInfoMap;
pub use connection::{ConnectionInfo, PlayerInfo};
pub use event::{
    AllowedCarsChanged, AllowedModsChanged, Connected, ConnectionDetails, Disconnected,
    LayoutChanged, MultiplayerJoined, MultiplayerLeft, PlayerJoined, PlayerLeft,
    PlayerTeleportedToPits, Renamed, SessionEnded, SessionStarted, TakingOver, TrackChanged,
    VehicleSelected, VersionReceived, WorldEvent,
};
pub use game::{
    GameInfo, GridMode, Month, MultiplayerState, SessionKind, TimeDemoPreset, TimeSet, VersionInfo,
};
use race::RaceState;
pub use race::{
    DriverRecord, EntrantId, EntrantState, FinishStatus, LapRecord, PitRecord, RaceEvent,
};

#[derive(Default)]
pub(crate) struct WorldInner {
    // Presence:
    pub(crate) connections: HashMap<ConnectionId, ConnectionInfo>,
    players: MultiIndexPlayerInfoMap,
    /// Survives `Cnl`: maps LFS.net username to last seen display name.
    last_known_names: HashMap<String, String>,

    /// Game state snapshot, including the current `session_kind`.
    pub(crate) game: GameInfo,

    /// Race tracking state.
    pub(crate) race: RaceState,

    /// When `true`, a joining player first attempts to resume a prior
    /// disconnected entrant (matched by LFS.net username) before falling back
    /// to a fresh entrant. Set once at construction.
    pub(crate) rejoin: bool,
}

impl WorldInner {
    fn apply_ncn(&mut self, ncn: &Ncn) -> ConnectionInfo {
        let info = ConnectionInfo {
            ucid: ncn.ucid,
            uname: ncn.uname.clone(),
            pname: ncn.pname.clone(),
            admin: ncn.admin,
            userid: None,
            ipaddress: None,
            selected_vehicle: None,
        };
        let _ = self
            .last_known_names
            .insert(ncn.uname.clone(), ncn.pname.clone());
        let _ = self.connections.insert(info.ucid, info.clone());
        info
    }

    fn apply_cnl(&mut self, cnl: &Cnl) -> (Option<ConnectionInfo>, Vec<PlayerInfo>) {
        let info = self.connections.remove(&cnl.ucid);
        let left = self.players.remove_by_ucid(&cnl.ucid);
        (info, left)
    }

    fn apply_nci(&mut self, nci: &Nci) -> Option<ConnectionInfo> {
        let conn = self.connections.get_mut(&nci.ucid)?;
        conn.userid = Some(nci.userid);
        conn.ipaddress = Some(nci.ipaddress);
        Some(conn.clone())
    }

    fn apply_slc(&mut self, slc: &Slc) -> Option<Vehicle> {
        let conn = self.connections.get_mut(&slc.ucid)?;
        conn.selected_vehicle = Some(slc.cname);
        Some(slc.cname)
    }

    fn apply_cpr(&mut self, cpr: &Cpr) -> Option<(ConnectionId, String, String)> {
        let conn = self.connections.get_mut(&cpr.ucid)?;
        conn.pname = cpr.pname.clone();
        let uname = conn.uname.clone();
        let _ = self
            .last_known_names
            .insert(uname.clone(), cpr.pname.clone());
        Some((cpr.ucid, uname, cpr.pname.clone()))
    }

    fn apply_npl(&mut self, npl: &Npl) -> Option<PlayerInfo> {
        let player = PlayerInfo {
            plid: npl.plid,
            ucid: npl.ucid,
            vehicle: npl.cname,
            ptype: npl.ptype,
            flags: npl.flags,
            in_pitlane: false,
            pname: npl.pname.clone(),
        };
        // Upsert: LFS re-announces existing players (e.g. in reply to the
        // `TINY_NPL` that `STARTUP_REQUESTS`/`SESSION_REQUESTS` issue), so the
        // same `plid` can arrive again. The `plid` index is unique, so a blind
        // `insert` would panic - replace any existing entry instead. We still
        // return `Some` for a re-announce: after a session restart the race
        // state is cleared while this map persists, and the race layer must see
        // the join to recreate the entrant (it dedupes via its own `live` map).
        let _ = self.players.remove_by_plid(&npl.plid);
        let _ = self.players.insert(player.clone());
        if npl.nump == 0 { None } else { Some(player) }
    }

    fn apply_pll(&mut self, pll: &Pll) -> Option<PlayerInfo> {
        self.players.remove_by_plid(&pll.plid)
    }

    fn apply_toc(&mut self, toc: &Toc) -> Option<(PlayerInfo, PlayerInfo)> {
        let before = self.players.get_by_plid(&toc.plid)?.clone();
        let after = self
            .players
            .modify_by_plid(&toc.plid, |p| p.ucid = toc.newucid)?
            .clone();
        Some((before, after))
    }

    fn apply_pfl(&mut self, pfl: &Pfl) {
        let _ = self
            .players
            .modify_by_plid(&pfl.plid, |p| p.flags = pfl.flags);
    }

    fn apply_pla(&mut self, pla: &Pla) {
        let _ = self.players.modify_by_plid(&pla.plid, |p| {
            if pla.entered_pitlane() {
                p.in_pitlane = true;
            }
            if pla.exited_pitlane() {
                p.in_pitlane = false;
            }
        });
    }

    fn apply_tiny_clr(&mut self, tiny: &Tiny) {
        if matches!(tiny.subt, TinyType::Clr) {
            self.players.clear();
        }
    }

    fn apply_sta(&mut self, sta: &Sta) -> (bool, bool, Option<Track>, Track) {
        let was_in_session = self.game.session_kind.is_some();
        let prev_track = self.game.track;
        self.game.session_kind = match sta.raceinprog {
            RaceInProgress::No => None,
            RaceInProgress::Racing => Some(SessionKind::Race {
                laps: sta.racelaps,
                flags: RaceFlags::empty(),
            }),
            RaceInProgress::Qualifying => Some(SessionKind::Qualifying {
                duration: Duration::from_secs(sta.qualmins as u64 * 60),
                flags: RaceFlags::empty(),
            }),
            _ => self.game.session_kind,
        };
        self.game.track = Some(sta.track);
        self.game.weather = Some(sta.weather);
        self.game.wind = Some(sta.wind);
        self.game.flags = sta.flags;
        let now_in_session = self.game.session_kind.is_some();
        (was_in_session, now_in_session, prev_track, sta.track)
    }

    fn apply_rst(&mut self, rst: &Rst) -> Option<SessionKind> {
        if rst.reqi.0 != 0 {
            return None;
        }
        self.game.track = Some(rst.track);
        self.game.weather = Some(rst.weather);
        self.game.wind = Some(rst.wind);
        let kind = if rst.qualmins > 0 {
            SessionKind::Qualifying {
                duration: Duration::from_secs(rst.qualmins as u64 * 60),
                flags: rst.flags,
            }
        } else if matches!(rst.racelaps, RaceLaps::Practice) {
            SessionKind::Practice
        } else if matches!(rst.racelaps, RaceLaps::Untimed) {
            SessionKind::Untimed
        } else {
            SessionKind::Race {
                laps: rst.racelaps,
                flags: rst.flags,
            }
        };
        self.game.rst_count = self.game.rst_count.wrapping_add(1);
        Some(kind)
    }

    fn apply_allowed_cars(&mut self, cars: &PlcAllowedCarsSet) -> bool {
        if self.game.allowed_cars.as_ref() == Some(cars) {
            return false;
        }
        self.game.allowed_cars = Some(cars.clone());
        true
    }

    fn apply_allowed_mods(&mut self, mods: &[Vehicle]) -> bool {
        if self.game.allowed_mods.as_slice() == mods {
            return false;
        }
        self.game.allowed_mods = mods.to_vec();
        true
    }

    fn apply_version(&mut self, ver: &Ver) {
        self.game.version = Some(VersionInfo {
            product: ver.product.clone(),
            version: ver.version.clone(),
        });
    }

    fn apply_axi(&mut self, axi: &Axi) -> (Option<String>, Option<String>) {
        let prev = self.game.layout.clone();
        self.game.layout = axi.lname.clone();
        self.game.axi_count = self.game.axi_count.wrapping_add(1);
        (prev, axi.lname.clone())
    }

    fn apply_ism(&mut self, ism: &Ism) -> (MultiplayerState, MultiplayerState) {
        let prev = self.game.multiplayer.clone();
        self.game.multiplayer = match ism.hname.as_deref() {
            None | Some("") => MultiplayerState::Local,
            Some(name) => MultiplayerState::Multiplayer {
                host_name: name.to_owned(),
                is_host: ism.host,
            },
        };
        (prev, self.game.multiplayer.clone())
    }

    fn apply_tiny_axc(&mut self, tiny: &Tiny) -> Option<String> {
        if !matches!(tiny.subt, TinyType::Axc) {
            return None;
        }
        self.game.layout.take()
    }
}

/// Apply one packet to `inner`, pushing any resulting [`WorldEvent`]s.
///
/// A single match over the packet type drives all three state mirrors
/// (connections/players, game state, race tracking) in one pass. Race-relevant
/// connection and session packets call into [`RaceState`] inline so race events
/// are interleaved in causal order with the presence/game events that triggered
/// them.
fn dispatch(inner: &mut WorldInner, packet: &insim::Packet, events: &mut Vec<WorldEvent>) {
    use insim::Packet;

    // Wrap a race-handler call, pushing each emitted [`RaceEvent`]. The handlers
    // live on `WorldInner` and read `session_kind`/`connections` themselves.
    macro_rules! push_race {
        ($call:expr) => {{
            for re in $call {
                events.push(WorldEvent::Race(re));
            }
        }};
    }

    // Deferred reset: a session-starting `Rst` marks the prior session's race
    // data for clearing but leaves it intact so `SessionEnded` consumers can
    // read the final results. Clear it now, before applying the first packet of
    // the new session.
    if std::mem::take(&mut inner.race.pending_reset) {
        inner.race.clear_for_session();
    }

    match packet {
        Packet::Ncn(ncn) => {
            let info = inner.apply_ncn(ncn);
            events.push(WorldEvent::Connected(Connected(info)));
        },
        Packet::Cnl(cnl) => {
            let (info, players) = inner.apply_cnl(cnl);
            for p in players {
                push_race!(inner.on_player_left(&p));
                events.push(WorldEvent::PlayerLeft(PlayerLeft(p)));
            }
            events.push(WorldEvent::Disconnected(Disconnected {
                ucid: cnl.ucid,
                info,
            }));
        },
        Packet::Nci(nci) => {
            if let Some(info) = inner.apply_nci(nci) {
                events.push(WorldEvent::ConnectionDetails(ConnectionDetails(info)));
            }
        },
        Packet::Slc(slc) => {
            if let Some(vehicle) = inner.apply_slc(slc) {
                events.push(WorldEvent::VehicleSelected(VehicleSelected {
                    ucid: slc.ucid,
                    vehicle,
                }));
            }
        },
        Packet::Cpr(cpr) => {
            if let Some((ucid, uname, new_pname)) = inner.apply_cpr(cpr) {
                events.push(WorldEvent::Renamed(Renamed {
                    ucid,
                    uname,
                    new_pname,
                }));
            }
        },
        Packet::Npl(npl) => {
            if let Some(info) = inner.apply_npl(npl) {
                push_race!(inner.on_player_joined(&info));
                events.push(WorldEvent::PlayerJoined(PlayerJoined(info)));
            }
        },
        Packet::Pll(pll) => {
            if let Some(info) = inner.apply_pll(pll) {
                push_race!(inner.on_player_left(&info));
                events.push(WorldEvent::PlayerLeft(PlayerLeft(info)));
            }
        },
        Packet::Toc(toc) => {
            if let Some((before, after)) = inner.apply_toc(toc) {
                push_race!(inner.on_taking_over(&before, &after));
                events.push(WorldEvent::TakingOver(TakingOver { before, after }));
            }
        },
        Packet::Pfl(pfl) => inner.apply_pfl(pfl),
        Packet::Pla(pla) => inner.apply_pla(pla),
        Packet::Plp(plp) => {
            // Presence-level and race-level views of the same tele-pit.
            if let Some(p) = inner.players.get_by_plid(&plp.plid).cloned() {
                events.push(WorldEvent::PlayerTeleportedToPits(PlayerTeleportedToPits(
                    p,
                )));
            }
            push_race!(inner.apply_telepit(plp));
        },
        Packet::Tiny(tiny) => {
            inner.apply_tiny_clr(tiny);
            if let Some(prev) = inner.apply_tiny_axc(tiny) {
                events.push(WorldEvent::LayoutChanged(LayoutChanged {
                    from: Some(prev),
                    to: None,
                }));
            }
        },
        Packet::Sta(sta) => {
            let (was_in_session, now_in_session, prev_track, new_track) = inner.apply_sta(sta);
            if was_in_session && !now_in_session {
                events.push(WorldEvent::SessionEnded(SessionEnded));
            }
            if prev_track != Some(new_track) {
                events.push(WorldEvent::TrackChanged(TrackChanged {
                    from: prev_track,
                    to: new_track,
                }));
            }
        },
        Packet::Rst(rst) => {
            if let Some(kind) = inner.apply_rst(rst) {
                // A `Rst` while a session is already active (e.g. a `/restart`)
                // ends that session without passing through the lobby, so the
                // `Sta`-driven `SessionEnded` never fires. Surface it here -
                // before deferring the clear - so consumers can read the final
                // results just like on the `/end` path.
                if inner.game.session_kind.is_some() {
                    events.push(WorldEvent::SessionEnded(SessionEnded));
                }
                inner.game.session_kind = Some(kind);
                inner.race.pending_reset = true;
                events.push(WorldEvent::SessionStarted(SessionStarted { kind }));
            }
        },
        Packet::Axi(axi) => {
            let (prev_lname, new_lname) = inner.apply_axi(axi);
            if prev_lname != new_lname {
                events.push(WorldEvent::LayoutChanged(LayoutChanged {
                    from: prev_lname,
                    to: new_lname,
                }));
            }
        },
        Packet::Ism(ism) => {
            let (prev, new) = inner.apply_ism(ism);
            if prev != new {
                match new {
                    MultiplayerState::Multiplayer { host_name, is_host } => {
                        events.push(WorldEvent::MultiplayerJoined(MultiplayerJoined {
                            host_name,
                            is_host,
                        }));
                    },
                    MultiplayerState::Local => {
                        events.push(WorldEvent::MultiplayerLeft(MultiplayerLeft));
                    },
                }
            }
        },
        Packet::Small(small) => {
            if let SmallType::Alc(cars) = &small.subt
                && inner.apply_allowed_cars(cars)
            {
                events.push(WorldEvent::AllowedCarsChanged(AllowedCarsChanged {
                    cars: cars.clone(),
                }));
            }
        },
        Packet::Mal(mal) => {
            let mods: Vec<Vehicle> = mal.iter().copied().collect();
            if inner.apply_allowed_mods(&mods) {
                events.push(WorldEvent::AllowedModsChanged(AllowedModsChanged { mods }));
            }
        },
        Packet::Ver(ver) => {
            inner.apply_version(ver);
            events.push(WorldEvent::VersionReceived(VersionReceived {
                product: ver.product.clone(),
                version: ver.version.clone(),
            }));
        },
        Packet::Lap(v) => push_race!(inner.apply_lap(v)),
        Packet::Spx(v) => push_race!(inner.apply_split(v)),
        Packet::Fin(v) => push_race!(inner.apply_finish(v)),
        Packet::Res(v) => push_race!(inner.apply_result(v)),
        Packet::Pit(v) => push_race!(inner.apply_pit_stop(v)),
        Packet::Psf(v) => push_race!(inner.apply_pit_stop_finished(v)),
        Packet::Pen(v) => push_race!(inner.apply_penalty_changed(v)),
        Packet::Reo(v) => push_race!(inner.apply_grid_order(v)),
        _ => {},
    }
}

/// Unified state mirror combining connection/player tracking, game-state
/// mirroring, and race tracking.
///
/// State lives behind `Arc<RwLock<…>>`; clones are cheap and share the same
/// underlying maps.
///
/// Use [`World::new()`] for short-form races and qualifying, where every player
/// join is a fresh entrant. Use [`World::with_rejoin()`] for long-form races
/// (endurance / multi-hour) where a player may briefly disconnect and reconnect.
#[derive(Clone)]
pub struct World {
    inner: Arc<RwLock<WorldInner>>,
    /// Generation counter bumped after every [`apply_packet`](World::apply_packet).
    /// Waiters in [`wait_until`](World::wait_until) block on changes to this
    /// instead of polling.
    version: Arc<watch::Sender<u64>>,
}

impl World {
    fn from_inner(inner: WorldInner) -> Self {
        let (version, _) = watch::channel(0u64);
        Self {
            inner: Arc::new(RwLock::new(inner)),
            version: Arc::new(version),
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::from_inner(WorldInner::default())
    }
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let g = self.inner.read();
        f.debug_struct("World")
            .field("connections", &g.connections.len())
            .field("players", &g.players.len())
            .field("track", &g.game.track)
            .field("session_kind", &g.game.session_kind)
            .field("race_entrants", &g.race.entrants.len())
            .field("rejoin", &g.rejoin)
            .finish()
    }
}

impl World {
    /// Tiny requests to send once on connect to sync connection/player lists
    /// and game/session state. LFS does not send these automatically on connect.
    pub const STARTUP_REQUESTS: &[TinyType] = &[
        TinyType::Ncn,
        TinyType::Npl,
        TinyType::Sst,
        TinyType::Axi,
        TinyType::Ism,
        TinyType::Alc,
        TinyType::Mal,
    ];

    /// Tiny requests to re-send on each [`WorldEvent::SessionStarted`], since a
    /// new session can change the allowed cars/mods, the player list, and the
    /// starting grid order.
    pub const SESSION_REQUESTS: &[TinyType] =
        &[TinyType::Alc, TinyType::Mal, TinyType::Npl, TinyType::Reo];

    /// Create a new world with empty state. Every player join creates a fresh
    /// entrant - correct for sprints, qualifying, and practice.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new world in rejoin mode. When a player joins, the tracker
    /// first attempts to resume a prior disconnected entrant (matched by LFS.net
    /// username), falling back to a fresh entrant if none is found. Use this for
    /// endurance / multi-hour races where mid-race reconnects should not create
    /// phantom duplicate entrants.
    pub fn with_rejoin() -> Self {
        Self::from_inner(WorldInner {
            rejoin: true,
            ..Default::default()
        })
    }

    /// Apply one raw packet, update all internal state, and return any events.
    ///
    /// A single match over the packet type drives the connection/player, game,
    /// and race mirrors in one pass, inside one write-lock acquisition.
    pub fn apply_packet(&self, packet: &insim::Packet) -> Vec<WorldEvent> {
        let mut events = Vec::new();
        {
            let mut inner = self.inner.write();
            dispatch(&mut inner, packet, &mut events);
        }
        // Wake any `wait_until` waiters so they re-check their predicate.
        self.version.send_modify(|v| *v = v.wrapping_add(1));
        events
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
    pub fn connection(&self, ucid: ConnectionId) -> Option<ConnectionInfo> {
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

    /// Currently selected track, if known.
    pub fn track(&self) -> Option<Track> {
        self.inner.read().game.track
    }

    /// Currently loaded layout, if known.
    pub fn layout(&self) -> Option<String> {
        self.inner.read().game.layout.clone()
    }

    /// Weather identifier (0..=2 typically).
    pub fn weather(&self) -> Option<u8> {
        self.inner.read().game.weather
    }

    /// Wind conditions.
    pub fn wind(&self) -> Option<Wind> {
        self.inner.read().game.wind
    }

    /// Current session kind. `None` means lobby / no session active.
    pub fn session(&self) -> Option<SessionKind> {
        self.inner.read().game.session_kind
    }

    /// Overall game flags.
    pub fn flags(&self) -> StaFlags {
        self.inner.read().game.flags
    }

    /// Current multiplayer state.
    pub fn multiplayer(&self) -> MultiplayerState {
        self.inner.read().game.multiplayer.clone()
    }

    /// Server's allowed-cars set, if a `Small`/`Alc` has been received.
    pub fn allowed_cars(&self) -> Option<PlcAllowedCarsSet> {
        self.inner.read().game.allowed_cars.clone()
    }

    /// Server's allowed-mods list, if a `Mal` has been received.
    pub fn allowed_mods(&self) -> Vec<Vehicle> {
        self.inner.read().game.allowed_mods.clone()
    }

    /// Version information about the connected LFS instance.
    pub fn version(&self) -> Option<VersionInfo> {
        self.inner.read().game.version.clone()
    }

    /// Snapshot of the current game state as a [`GameInfo`].
    pub fn game_info(&self) -> GameInfo {
        self.inner.read().game.clone()
    }

    /// Number of `Axi` (autocross info) packets received so far. Snapshot this
    /// before issuing a layout command, then [`wait_until`](Self::wait_until)
    /// the count changes to detect the confirming `Axi`.
    pub fn axi_count(&self) -> u64 {
        self.inner.read().game.axi_count
    }

    /// Number of `Rst` (race start) packets received so far. Snapshot this
    /// before issuing a restart, then [`wait_until`](Self::wait_until) the count
    /// changes to detect the confirming `Rst`.
    pub fn rst_count(&self) -> u64 {
        self.inner.read().game.rst_count
    }

    /// Block until `f` returns `Some`, re-evaluating it each time state changes
    /// (i.e. after each [`apply_packet`](Self::apply_packet)), or until `cancel`
    /// fires (in which case `None`).
    ///
    /// This is edge-triggered with re-check, not polled: the receiver is created
    /// before the first evaluation, so a state change racing the predicate check
    /// is never missed.
    pub async fn wait_until<T>(
        &self,
        cancel: CancellationToken,
        f: impl Fn(&World) -> Option<T>,
    ) -> Option<T> {
        let mut rx = self.version.subscribe();
        loop {
            if let Some(v) = f(self) {
                return Some(v);
            }
            tokio::select! {
                biased;
                _ = cancel.cancelled() => return None,
                res = rx.changed() => res.ok()?,
            }
        }
    }

    /// The current session fastest lap: entrant, player ID, and time.
    pub fn fastest_lap(&self) -> Option<(EntrantId, PlayerId, Duration)> {
        self.inner.read().race.fastest_lap
    }

    /// Whether a race session (not qualifying or practice) is currently active.
    pub fn race_active(&self) -> bool {
        self.inner
            .read()
            .game
            .session_kind
            .is_some_and(|k| k.is_race())
    }

    /// Look up an entrant by their stable [`EntrantId`].
    pub fn entrant(&self, id: EntrantId) -> Option<EntrantState> {
        self.inner.read().race.entrants.get(&id).cloned()
    }

    /// Look up the currently-live entrant for a [`PlayerId`].
    pub fn entrant_by_plid(&self, plid: PlayerId) -> Option<EntrantState> {
        let g = self.inner.read();
        let id = g.race.live.get(&plid)?;
        g.race.entrants.get(id).cloned()
    }

    /// Snapshot of all entrants (racing, finished, and DNF).
    pub fn entrants(&self) -> Vec<EntrantState> {
        self.inner.read().race.entrants.values().cloned().collect()
    }

    /// Snapshot of entrants currently on track.
    pub fn live_entrants(&self) -> Vec<EntrantState> {
        let g = self.inner.read();
        g.race
            .live
            .values()
            .filter_map(|id| g.race.entrants.get(id).cloned())
            .collect()
    }
}

#[cfg(test)]
mod emission_tests {
    use insim::{
        core::track::Track,
        insim::{Axi, RaceInProgress, RaceLaps, Rst, Sta},
    };

    use super::{World, WorldEvent};

    fn count(events: &[WorldEvent], pred: impl Fn(&WorldEvent) -> bool) -> usize {
        events.iter().filter(|e| pred(e)).count()
    }

    #[test]
    fn rst_emits_session_started() {
        let world = World::new();
        let events = world.apply_packet(&insim::Packet::Rst(Rst {
            racelaps: RaceLaps::Laps(5),
            ..Default::default()
        }));
        assert_eq!(
            count(&events, |e| matches!(e, WorldEvent::SessionStarted(_))),
            1,
            "SessionStarted should fire once on Rst"
        );
    }

    #[test]
    fn sta_emits_session_ended_on_leaving_race() {
        let world = World::new();

        // Racing in progress: no SessionEnded.
        let events = world.apply_packet(&insim::Packet::Sta(Sta {
            raceinprog: RaceInProgress::Racing,
            ..Default::default()
        }));
        assert_eq!(
            count(&events, |e| matches!(e, WorldEvent::SessionEnded(_))),
            0
        );

        // Back to no race: SessionEnded fires once.
        let events = world.apply_packet(&insim::Packet::Sta(Sta {
            raceinprog: RaceInProgress::No,
            ..Default::default()
        }));
        assert_eq!(
            count(&events, |e| matches!(e, WorldEvent::SessionEnded(_))),
            1,
            "SessionEnded should fire once on Racing -> No transition"
        );
    }

    #[test]
    fn sta_emits_track_changed_on_track_field_change() {
        let world = World::new();
        let track_a = Track::ALL[0];
        let track_b = *Track::ALL
            .iter()
            .find(|t| **t != track_a)
            .expect("at least two tracks");

        let sta = |track| {
            insim::Packet::Sta(Sta {
                track,
                ..Default::default()
            })
        };
        let track_changes = |events: &[WorldEvent]| -> Vec<(Option<Track>, Track)> {
            events
                .iter()
                .filter_map(|e| match e {
                    WorldEvent::TrackChanged(t) => Some((t.from, t.to)),
                    _ => None,
                })
                .collect()
        };

        let changes = track_changes(&world.apply_packet(&sta(track_a)));
        assert_eq!(
            changes,
            vec![(None, track_a)],
            "first Sta emits TrackChanged"
        );

        let changes = track_changes(&world.apply_packet(&sta(track_a)));
        assert!(
            changes.is_empty(),
            "same track should not emit TrackChanged"
        );

        let changes = track_changes(&world.apply_packet(&sta(track_b)));
        assert_eq!(changes, vec![(Some(track_a), track_b)]);
    }

    #[test]
    fn axi_emits_layout_changed_on_layout_field_change() {
        let world = World::new();
        let layout_a = String::new();
        let layout_b = "test".to_string();

        let axi = |lname: &str| {
            insim::Packet::Axi(Axi {
                lname: Some(lname.to_string()),
                ..Default::default()
            })
        };
        let layout_changes = |events: &[WorldEvent]| -> Vec<(Option<String>, Option<String>)> {
            events
                .iter()
                .filter_map(|e| match e {
                    WorldEvent::LayoutChanged(l) => Some((l.from.clone(), l.to.clone())),
                    _ => None,
                })
                .collect()
        };

        let changes = layout_changes(&world.apply_packet(&axi(&layout_a)));
        assert_eq!(
            changes,
            vec![(None, Some(layout_a.clone()))],
            "first Axi emits LayoutChanged"
        );

        let changes = layout_changes(&world.apply_packet(&axi(&layout_a)));
        assert!(
            changes.is_empty(),
            "same layout should not emit LayoutChanged"
        );

        let changes = layout_changes(&world.apply_packet(&axi(&layout_b)));
        assert_eq!(changes, vec![(Some(layout_a), Some(layout_b))]);
    }
}
