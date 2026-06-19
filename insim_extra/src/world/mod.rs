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
        Axi, Cnl, Cpr, Ism, Mal, Nci, Ncn, Npl, Pfl, Pla, Plc, PlcAllowedCarsSet, Pll, RaceFlags,
        RaceInProgress, RaceLaps, Rst, Slc, SmallType, Sta, StaFlags, Tiny, TinyType, Toc, Ver,
    },
};
use parking_lot::RwLock;
use tokio_util::sync::CancellationToken;

use crate::util::host_command;

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
struct WorldInner {
    // Presence:
    connections: HashMap<ConnectionId, ConnectionInfo>,
    players: MultiIndexPlayerInfoMap,
    /// Survives `Cnl`: maps LFS.net username to last seen display name.
    last_known_names: HashMap<String, String>,

    /// Game state snapshot, including the current `session_kind`.
    game: GameInfo,

    /// Race tracking state.
    race: RaceState,
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
fn dispatch(
    inner: &mut WorldInner,
    packet: &insim::Packet,
    rejoin: bool,
    events: &mut Vec<WorldEvent>,
) {
    use insim::Packet;

    // Push every race event from a timing-packet handler, reading `session_kind`
    // first to avoid borrowing `inner` twice.
    macro_rules! race_timing {
        ($method:ident, $v:expr) => {{
            let sk = inner.game.session_kind;
            for re in inner.race.$method($v, sk) {
                events.push(WorldEvent::Race(re));
            }
        }};
    }

    match packet {
        Packet::Ncn(ncn) => {
            let info = inner.apply_ncn(ncn);
            events.push(WorldEvent::Connected(Connected(info)));
        },
        Packet::Cnl(cnl) => {
            let (info, players) = inner.apply_cnl(cnl);
            let sk = inner.game.session_kind;
            for p in players {
                for re in inner.race.on_player_left(&p, sk) {
                    events.push(WorldEvent::Race(re));
                }
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
                let sk = inner.game.session_kind;
                let WorldInner {
                    race, connections, ..
                } = inner;
                for re in race.on_player_joined(&info, connections, sk, rejoin) {
                    events.push(WorldEvent::Race(re));
                }
                events.push(WorldEvent::PlayerJoined(PlayerJoined(info)));
            }
        },
        Packet::Pll(pll) => {
            if let Some(info) = inner.apply_pll(pll) {
                let sk = inner.game.session_kind;
                for re in inner.race.on_player_left(&info, sk) {
                    events.push(WorldEvent::Race(re));
                }
                events.push(WorldEvent::PlayerLeft(PlayerLeft(info)));
            }
        },
        Packet::Toc(toc) => {
            if let Some((before, after)) = inner.apply_toc(toc) {
                let sk = inner.game.session_kind;
                let WorldInner {
                    race, connections, ..
                } = inner;
                for re in race.on_taking_over(&before, &after, connections, sk) {
                    events.push(WorldEvent::Race(re));
                }
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
            race_timing!(apply_telepit, plp);
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
                inner.game.session_kind = Some(kind);
                inner.race.clear_for_session();
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
        Packet::Lap(v) => race_timing!(apply_lap, v),
        Packet::Spx(v) => race_timing!(apply_split, v),
        Packet::Fin(v) => race_timing!(apply_finish, v),
        Packet::Res(v) => race_timing!(apply_result, v),
        Packet::Pit(v) => race_timing!(apply_pit_stop, v),
        Packet::Psf(v) => race_timing!(apply_pit_stop_finished, v),
        Packet::Pen(v) => race_timing!(apply_penalty_changed, v),
        Packet::Reo(v) => race_timing!(apply_grid_order, v),
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
    rejoin: bool,
}

impl Default for World {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(WorldInner::default())),
            rejoin: false,
        }
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
            .field("rejoin", &self.rejoin)
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
        Self {
            rejoin: true,
            ..Self::default()
        }
    }

    /// Apply one raw packet, update all internal state, and return any events.
    ///
    /// A single match over the packet type drives the connection/player, game,
    /// and race mirrors in one pass, inside one write-lock acquisition.
    pub fn apply_packet(&self, packet: &insim::Packet) -> Vec<WorldEvent> {
        let mut events = Vec::new();
        {
            let mut inner = self.inner.write();
            dispatch(&mut inner, packet, self.rejoin, &mut events);
        }
        events
    }

    // ── Presence query methods ────────────────────────────────────────────────

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

    /// Currently selected track, if known.
    pub fn current_track(&self) -> Option<Track> {
        self.inner.read().game.track
    }

    /// Currently loaded layout, if known.
    pub fn current_layout(&self) -> Option<String> {
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

    async fn wait_for_game<F: Fn(&GameInfo) -> bool>(
        &self,
        predicate: F,
        poll_interval: Duration,
        cancel: CancellationToken,
    ) -> Option<()> {
        let mut interval = tokio::time::interval(poll_interval);
        loop {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => return None,
                _ = interval.tick() => {
                    if predicate(&self.game_info()) {
                        return Some(());
                    }
                }
            }
        }
    }

    /// Wait until state is populated from at least one `Sta` packet - i.e.
    /// the current track is known.
    pub async fn wait_for_known_state(&self, cancel: CancellationToken) -> Option<()> {
        self.wait_for_game(
            |info| info.current_track().is_some(),
            Duration::from_millis(100),
            cancel,
        )
        .await
    }

    /// Wait until the game is no longer in progress.
    pub async fn wait_for_end(&self, cancel: CancellationToken) -> Option<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        loop {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => return None,
                _ = interval.tick() => {
                    if self.inner.read().game.session_kind.is_none() {
                        return Some(());
                    }
                }
            }
        }
    }

    /// Wait until the given track is loaded and no session is active
    /// (selection screen, not yet racing).
    pub async fn wait_for_track(&self, track: Track, cancel: CancellationToken) -> Option<()> {
        self.wait_for_game(
            move |info| info.current_track() == Some(&track),
            Duration::from_millis(500),
            cancel,
        )
        .await
    }

    /// Wait until a race session is in progress.
    pub async fn wait_for_racing(&self, cancel: CancellationToken) -> Option<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        loop {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => return None,
                _ = interval.tick() => {
                    if matches!(self.inner.read().game.session_kind, Some(SessionKind::Race { .. })) {
                        return Some(());
                    }
                }
            }
        }
    }

    /// Wait for a specific layout to be loaded.
    pub async fn wait_for_layout(&self, layout: String, cancel: CancellationToken) -> Option<()> {
        self.wait_for_game(
            move |info| {
                info.current_layout()
                    .map(|l| l.as_str() == layout.as_str())
                    .unwrap_or(false)
            },
            Duration::from_millis(500),
            cancel,
        )
        .await
    }

    /// Wait for any `Axi` packet to be received.
    pub async fn wait_for_any_axi(&self, cancel: CancellationToken) -> Option<()> {
        let before = self.inner.read().game.axi_count;
        self.wait_for_game(
            move |info| info.axi_count != before,
            Duration::from_millis(100),
            cancel,
        )
        .await
    }

    /// Wait for any `Rst` packet to be received, indicating a race or
    /// qualifying session has started.
    pub async fn wait_for_any_rst(&self, cancel: CancellationToken) -> Option<()> {
        let before = self.inner.read().game.rst_count;
        self.wait_for_game(
            move |info| info.rst_count != before,
            Duration::from_millis(100),
            cancel,
        )
        .await
    }

    /// `/end` - finish the current race.
    pub fn end(&self) -> insim::Packet {
        host_command("/end")
    }

    /// `/clear` - remove all connections from the server.
    pub fn clear(&self) -> insim::Packet {
        host_command("/clear")
    }

    /// `/track {track}` - load a different track.
    pub fn change_track(&self, track: Track) -> insim::Packet {
        host_command(format!("/track {track}"))
    }

    /// Change race length. Maps onto `/laps`, `/hours`, or `/laps no`.
    pub fn change_laps(&self, laps: RaceLaps) -> insim::Packet {
        let cmd = match laps {
            RaceLaps::Untimed => "/laps no".to_string(),
            RaceLaps::Hours(h) => format!("/hours {h}"),
            other => format!("/laps {}", Into::<u8>::into(other)),
        };
        host_command(cmd)
    }

    /// `/wind {wind}` - set wind strength (0..=2 typically).
    pub fn change_wind(&self, wind: u8) -> insim::Packet {
        host_command(format!("/wind {wind}"))
    }

    /// `/axclear` - clear the autocross layout.
    pub fn ax_clear(&self) -> insim::Packet {
        host_command("/axclear")
    }

    /// `/axload {layout}` - load an autocross layout by name.
    pub fn ax_load(&self, layout: impl Into<String>) -> insim::Packet {
        host_command(format!("/axload {}", layout.into()))
    }

    /// `/restart` - start a race.
    pub fn restart(&self) -> insim::Packet {
        host_command("/restart")
    }

    /// `/qualify` - start qualifying.
    pub fn qualify(&self) -> insim::Packet {
        host_command("/qualify")
    }

    /// `/reinit` - full restart, kicks all connections.
    pub fn reinit(&self) -> insim::Packet {
        host_command("/reinit")
    }

    /// `/weather {weather}` - set weather/lighting.
    pub fn change_weather(&self, weather: u8) -> insim::Packet {
        host_command(format!("/weather {weather}"))
    }

    /// `/qual {minutes}` - set qualifying duration. `0` = no qualifying.
    pub fn change_qual(&self, minutes: u8) -> insim::Packet {
        host_command(format!("/qual {minutes}"))
    }

    /// `/time` - report the current in-game time status.
    pub fn time_status(&self) -> insim::Packet {
        host_command("/time")
    }

    /// `/time live` - switch to live (real-world) time.
    pub fn time_live(&self) -> insim::Packet {
        host_command("/time live")
    }

    /// `/time offset [days] [HH:MM]` - shift in-game time by an offset.
    pub fn time_offset(&self, days: Option<i32>, minutes: Option<i32>) -> insim::Packet {
        let mut cmd = String::from("/time offset");
        if let Some(d) = days {
            let sign = if d < 0 { '-' } else { '+' };
            cmd.push_str(&format!(" {sign}{}", d.unsigned_abs()));
        }
        if let Some(m) = minutes {
            let sign = if m < 0 { '-' } else { '+' };
            let abs = m.unsigned_abs();
            cmd.push_str(&format!(" {sign}{}:{:02}", abs / 60, abs % 60));
        }
        host_command(cmd)
    }

    /// `/time set [DD Mon] [HH:MM] [utc±offset]` - set in-game time explicitly.
    pub fn time_set(&self, params: TimeSet) -> insim::Packet {
        let mut cmd = String::from("/time set");
        if let Some((day, month)) = params.date {
            cmd.push_str(&format!(" {day} {month}"));
        }
        if let Some((hour, minute)) = params.time {
            cmd.push_str(&format!(" {hour:02}:{minute:02}"));
        }
        if let Some(off) = params.utc_offset {
            let sign = if off < 0 { '-' } else { '+' };
            cmd.push_str(&format!(" utc{sign}{}", off.unsigned_abs()));
        }
        host_command(cmd)
    }

    /// `/time mul {0..=240}` - set the time multiplier (set-time mode only).
    pub fn time_multiplier(&self, factor: u8) -> insim::Packet {
        host_command(format!("/time mul {factor}"))
    }

    /// `/time demo {preset}` - activate a demo time-of-day preset.
    pub fn time_demo(&self, preset: TimeDemoPreset) -> insim::Packet {
        host_command(format!("/time demo {preset}"))
    }

    /// `/pit_all` - send every player to the pits.
    pub fn pit_all(&self) -> insim::Packet {
        host_command("/pit_all")
    }

    /// `/spec_all` - spectate all players.
    pub fn spec_all(&self) -> insim::Packet {
        host_command("/spec_all")
    }

    /// `/grid open|self|lock` - set who can modify the grid in the game setup screen.
    pub fn change_grid(&self, mode: GridMode) -> insim::Packet {
        host_command(format!("/grid {mode}"))
    }

    /// `/grid real yes` / `/grid real no` - allow or disallow real players joining.
    pub fn change_grid_real(&self, allow: bool) -> insim::Packet {
        host_command(if allow {
            "/grid real yes"
        } else {
            "/grid real no"
        })
    }

    /// `/grid ai yes` / `/grid ai no` - allow or disallow AI players joining.
    pub fn change_grid_ai(&self, allow: bool) -> insim::Packet {
        host_command(if allow { "/grid ai yes" } else { "/grid ai no" })
    }

    /// `/flood yes` / `/flood no` - switch floodlights on or off.
    pub fn change_flood(&self, on: bool) -> insim::Packet {
        host_command(if on { "/flood yes" } else { "/flood no" })
    }

    /// Apply vehicle restrictions server-wide (ucid = `ConnectionId::ALL`).
    pub fn restrict_vehicles(&self, vehicles: &[Vehicle]) -> Vec<insim::Packet> {
        let mut mal = Mal::default();
        let cars = if vehicles.is_empty() {
            PlcAllowedCarsSet::all()
        } else {
            let mut cars = PlcAllowedCarsSet::default();
            for v in vehicles {
                match v {
                    Vehicle::Mod(_) => {
                        let _ = mal.insert(*v);
                    },
                    _ => {
                        let _ = cars.insert(*v);
                    },
                }
            }
            cars
        };
        vec![
            insim::Packet::from(Plc {
                cars,
                ucid: ConnectionId::ALL,
                ..Plc::default()
            }),
            insim::Packet::from(mal),
        ]
    }

    /// Returns an `/unban` packet.
    pub fn unban(&self, uname: impl Into<String>) -> insim::Packet {
        host_command(format!("/unban {}", uname.into()))
    }

    /// Returns a `/kick` packet for the given UCID, or `None` if not found.
    pub fn kick(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        Some(self.get(ucid)?.kick())
    }

    /// Returns a `/ban` packet. `ban_days = 0` means 12 hours (LFS convention).
    pub fn ban(&self, ucid: ConnectionId, ban_days: u32) -> Option<insim::Packet> {
        Some(self.get(ucid)?.ban(ban_days))
    }

    /// Returns a `/spec` packet for the given UCID, or `None` if not found.
    pub fn spec(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        Some(self.get(ucid)?.spec())
    }

    /// Returns a `/pitlane` packet for the given UCID, or `None` if not found.
    pub fn pitlane(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        Some(self.get(ucid)?.pitlane())
    }

    /// Returns a `/p_clear` packet for the given UCID, or `None` if not found.
    pub fn clear_penalty(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        Some(self.get(ucid)?.clear_penalty())
    }

    /// Returns a penalty packet for the given UCID.
    pub fn give_penalty(
        &self,
        ucid: ConnectionId,
        penalty: insim::insim::PenaltyInfo,
    ) -> Option<insim::Packet> {
        self.get(ucid)?.give_penalty(penalty)
    }

    /// Returns the packets needed to set and display a Race Control Message.
    pub fn send_rcm(&self, message: &str, ucid: ConnectionId) -> Vec<insim::Packet> {
        if ucid == ConnectionId::ALL {
            return vec![
                host_command(format!("/rcm {message}")),
                host_command("/rcm_all"),
            ];
        }
        self.get(ucid)
            .map(|conn| conn.send_rcm(message))
            .unwrap_or_default()
    }

    /// Returns the packets needed to clear a Race Control Message.
    pub fn clear_rcm(&self, ucid: ConnectionId) -> Option<insim::Packet> {
        if ucid == ConnectionId::ALL {
            return Some(host_command("/rcc_all"));
        }
        Some(self.get(ucid)?.clear_rcm())
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
