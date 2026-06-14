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
//!             WorldEvent::Presence(pe) => { /* ... */ }
//!             WorldEvent::Game(ge) => { /* ... */ }
//!             WorldEvent::Race(re) => { /* ... */ }
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

use crate::{
    game::{
        GameEvent, GameInfo, GridMode, MultiplayerState, SessionKind, SessionState, TimeDemoPreset,
        TimeSet, VersionInfo,
    },
    presence::{ConnectionInfo, MultiIndexPlayerInfoMap, PlayerInfo, PresenceEvent},
    race::{RaceEvent, RaceTracker},
    util::host_command,
};

/// Aggregate event produced by [`World::apply_packet`].
#[derive(Debug, Clone)]
pub enum WorldEvent {
    /// A presence change (connection joined/left, player joined/left, etc.).
    Presence(PresenceEvent),
    /// A game-state change (session started/ended, track changed, etc.).
    Game(GameEvent),
    /// A race event (entrant joined, lap completed, finished, etc.).
    Race(RaceEvent),
}

#[derive(Default)]
struct WorldInner {
    // From PresenceInner:
    connections: HashMap<ConnectionId, ConnectionInfo>,
    players: MultiIndexPlayerInfoMap,
    /// Survives `Cnl`: maps LFS.net username to last seen display name.
    last_known_names: HashMap<String, String>,

    // From GameInfo:
    track: Option<Track>,
    layout: Option<String>,
    weather: Option<u8>,
    wind: Option<Wind>,
    session: SessionState,
    flags: StaFlags,
    multiplayer: MultiplayerState,
    /// Server's allowed-cars set, from a `Small`/`Alc` reply.
    allowed_cars: Option<PlcAllowedCarsSet>,
    /// Server's allowed-mods list, from a `Mal` packet.
    allowed_mods: Vec<Vehicle>,
    /// Version information, from a `Ver` packet.
    version: Option<VersionInfo>,
    /// Incremented each time an `Axi` packet is applied.
    axi_count: u64,
    /// Incremented each time an `Rst` packet is applied.
    rst_count: u64,
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

    fn apply_slc(&mut self, slc: &Slc) -> bool {
        if let Some(conn) = self.connections.get_mut(&slc.ucid) {
            conn.selected_vehicle = Some(slc.cname);
            true
        } else {
            false
        }
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
        let was_in_session = matches!(
            self.session,
            SessionState::Racing { .. } | SessionState::Qualifying { .. }
        );
        let prev_track = self.track;
        self.session = match sta.raceinprog {
            RaceInProgress::No => SessionState::Lobby,
            RaceInProgress::Racing => SessionState::Racing {
                laps: sta.racelaps,
                flags: RaceFlags::empty(),
            },
            RaceInProgress::Qualifying => SessionState::Qualifying {
                duration: Duration::from_secs(sta.qualmins as u64 * 60),
                flags: RaceFlags::empty(),
            },
            _ => SessionState::Unknown,
        };
        self.track = Some(sta.track);
        self.weather = Some(sta.weather);
        self.wind = Some(sta.wind);
        self.flags = sta.flags;
        let now_in_session = matches!(
            self.session,
            SessionState::Racing { .. } | SessionState::Qualifying { .. }
        );
        (was_in_session, now_in_session, prev_track, sta.track)
    }

    fn apply_rst(&mut self, rst: &Rst) -> Option<SessionKind> {
        if rst.reqi.0 != 0 {
            return None;
        }
        self.track = Some(rst.track);
        self.weather = Some(rst.weather);
        self.wind = Some(rst.wind);
        let kind = if rst.qualmins > 0 {
            self.session = SessionState::Qualifying {
                duration: Duration::from_secs(rst.qualmins as u64 * 60),
                flags: rst.flags,
            };
            SessionKind::Qualifying
        } else if let RaceLaps::Practice | RaceLaps::Untimed = rst.racelaps {
            if matches!(rst.racelaps, RaceLaps::Untimed) {
                SessionKind::Untimed
            } else {
                SessionKind::Practice
            }
        } else {
            self.session = SessionState::Racing {
                laps: rst.racelaps,
                flags: rst.flags,
            };
            SessionKind::Race
        };
        self.rst_count = self.rst_count.wrapping_add(1);
        Some(kind)
    }

    fn apply_allowed_cars(&mut self, cars: &PlcAllowedCarsSet) -> bool {
        if self.allowed_cars.as_ref() == Some(cars) {
            return false;
        }
        self.allowed_cars = Some(cars.clone());
        true
    }

    fn apply_allowed_mods(&mut self, mods: &[Vehicle]) -> bool {
        if self.allowed_mods.as_slice() == mods {
            return false;
        }
        self.allowed_mods = mods.to_vec();
        true
    }

    fn apply_version(&mut self, ver: &Ver) {
        self.version = Some(VersionInfo {
            product: ver.product.clone(),
            version: ver.version.clone(),
        });
    }

    fn apply_axi(&mut self, axi: &Axi) -> (Option<String>, Option<String>) {
        let prev = self.layout.clone();
        self.layout = axi.lname.clone();
        self.axi_count = self.axi_count.wrapping_add(1);
        (prev, axi.lname.clone())
    }

    fn apply_ism(&mut self, ism: &Ism) -> (MultiplayerState, MultiplayerState) {
        let prev = self.multiplayer.clone();
        self.multiplayer = match ism.hname.as_deref() {
            None | Some("") => MultiplayerState::Local,
            Some(name) => MultiplayerState::Multiplayer {
                host_name: name.to_owned(),
                is_host: ism.host,
            },
        };
        (prev, self.multiplayer.clone())
    }

    fn apply_tiny_axc(&mut self, tiny: &Tiny) -> Option<String> {
        if !matches!(tiny.subt, TinyType::Axc) {
            return None;
        }
        self.layout.take()
    }
}

fn apply_presence_packet(inner: &mut WorldInner, packet: &insim::Packet) -> Vec<PresenceEvent> {
    match packet {
        insim::Packet::Ncn(ncn) => vec![PresenceEvent::Connected(inner.apply_ncn(ncn))],
        insim::Packet::Cnl(cnl) => {
            let (info, players) = inner.apply_cnl(cnl);
            players
                .into_iter()
                .map(PresenceEvent::PlayerLeft)
                .chain(std::iter::once(PresenceEvent::Disconnected {
                    ucid: cnl.ucid,
                    info,
                }))
                .collect()
        },
        insim::Packet::Nci(nci) => inner
            .apply_nci(nci)
            .map(PresenceEvent::ConnectionDetails)
            .into_iter()
            .collect(),
        insim::Packet::Slc(slc) => {
            if inner.apply_slc(slc) {
                vec![PresenceEvent::VehicleSelected {
                    ucid: slc.ucid,
                    vehicle: slc.cname,
                }]
            } else {
                vec![]
            }
        },
        insim::Packet::Cpr(cpr) => inner
            .apply_cpr(cpr)
            .map(|(ucid, uname, new_pname)| PresenceEvent::Renamed {
                ucid,
                uname,
                new_pname,
            })
            .into_iter()
            .collect(),
        insim::Packet::Npl(npl) => inner
            .apply_npl(npl)
            .map(PresenceEvent::PlayerJoined)
            .into_iter()
            .collect(),
        insim::Packet::Pll(pll) => inner
            .apply_pll(pll)
            .map(PresenceEvent::PlayerLeft)
            .into_iter()
            .collect(),
        insim::Packet::Toc(toc) => inner
            .apply_toc(toc)
            .map(|(before, after)| PresenceEvent::TakingOver { before, after })
            .into_iter()
            .collect(),
        insim::Packet::Pfl(pfl) => {
            inner.apply_pfl(pfl);
            vec![]
        },
        insim::Packet::Pla(pla) => {
            inner.apply_pla(pla);
            vec![]
        },
        insim::Packet::Plp(plp) => {
            let player = inner.players.get_by_plid(&plp.plid).cloned();
            if let Some(p) = player {
                vec![PresenceEvent::PlayerTeleportedToPits(p)]
            } else {
                vec![]
            }
        },
        insim::Packet::Tiny(tiny) => {
            inner.apply_tiny_clr(tiny);
            vec![]
        },
        _ => vec![],
    }
}

fn apply_game_packet(inner: &mut WorldInner, packet: &insim::Packet) -> Vec<GameEvent> {
    match packet {
        insim::Packet::Sta(sta) => {
            let (was_in_session, now_in_session, prev_track, new_track) = inner.apply_sta(sta);
            let mut events = Vec::new();
            if was_in_session && !now_in_session {
                events.push(GameEvent::SessionEnded);
            }
            if prev_track != Some(new_track) {
                events.push(GameEvent::TrackChanged {
                    from: prev_track,
                    to: new_track,
                });
            }
            events
        },
        insim::Packet::Axi(axi) => {
            let (prev_lname, new_lname) = inner.apply_axi(axi);
            if prev_lname != new_lname {
                vec![GameEvent::LayoutChanged {
                    from: prev_lname,
                    to: new_lname,
                }]
            } else {
                vec![]
            }
        },
        insim::Packet::Ism(ism) => {
            let (prev, new) = inner.apply_ism(ism);
            if prev == new {
                return vec![];
            }
            match new {
                MultiplayerState::Multiplayer { host_name, is_host } => {
                    vec![GameEvent::MultiplayerJoined { host_name, is_host }]
                },
                MultiplayerState::Local => vec![GameEvent::MultiplayerLeft],
            }
        },
        insim::Packet::Tiny(tiny) => inner
            .apply_tiny_axc(tiny)
            .map(|prev| GameEvent::LayoutChanged {
                from: Some(prev),
                to: None,
            })
            .into_iter()
            .collect(),
        insim::Packet::Rst(rst) => {
            if let Some(kind) = inner.apply_rst(rst) {
                vec![GameEvent::SessionStarted { kind }]
            } else {
                vec![]
            }
        },
        insim::Packet::Small(small) => {
            if let SmallType::Alc(cars) = &small.subt {
                if inner.apply_allowed_cars(cars) {
                    vec![GameEvent::AllowedCarsChanged { cars: cars.clone() }]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        },
        insim::Packet::Mal(mal) => {
            let mods: Vec<Vehicle> = mal.iter().copied().collect();
            if inner.apply_allowed_mods(&mods) {
                vec![GameEvent::AllowedModsChanged { mods }]
            } else {
                vec![]
            }
        },
        insim::Packet::Ver(ver) => {
            inner.apply_version(ver);
            vec![GameEvent::VersionReceived {
                product: ver.product.clone(),
                version: ver.version.clone(),
            }]
        },
        _ => vec![],
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
    race: RaceTracker,
    rejoin: bool,
}

impl Default for World {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(WorldInner::default())),
            race: RaceTracker::default(),
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
            .field("track", &g.track)
            .field("session", &g.session)
            .field("race", &self.race)
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

    /// Tiny requests to re-send on each [`GameEvent::SessionStarted`], since a
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
    /// Processing order:
    /// 1. Presence - connection and player events (write lock acquired once)
    /// 2. Each [`PresenceEvent`] is immediately routed into the race tracker
    ///    (no deferred cycle), producing [`RaceEvent`]s inline.
    /// 3. Game - session and track events (from the same write lock)
    /// 4. Each [`GameEvent`] is immediately routed into the race tracker.
    /// 5. Timing packets (`Lap`, `Spx`, `Fin`, `Res`, `Pit`, `Psf`, `Pen`,
    ///    `Plp`, `Reo`) are routed directly to the race tracker.
    pub fn apply_packet(&self, packet: &insim::Packet) -> Vec<WorldEvent> {
        let (presence_events, game_events) = {
            let mut inner = self.inner.write();
            let pe = apply_presence_packet(&mut inner, packet);
            let ge = apply_game_packet(&mut inner, packet);
            (pe, ge)
        };

        let mut events = Vec::new();

        for pe in presence_events {
            let race_events = if self.rejoin {
                if let PresenceEvent::PlayerJoined(ref info) = pe {
                    self.race.apply_player_rejoined(info)
                } else {
                    self.race.apply_presence_event(&pe)
                }
            } else {
                self.race.apply_presence_event(&pe)
            };
            for re in race_events {
                events.push(WorldEvent::Race(re));
            }
            events.push(WorldEvent::Presence(pe));
        }

        for ge in game_events {
            for re in self.race.apply_game_event(&ge) {
                events.push(WorldEvent::Race(re));
            }
            events.push(WorldEvent::Game(ge));
        }

        for re in self.race.apply_packet(packet) {
            events.push(WorldEvent::Race(re));
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

    // ── Game query methods ────────────────────────────────────────────────────

    /// Currently selected track, if known.
    pub fn current_track(&self) -> Option<Track> {
        self.inner.read().track
    }

    /// Currently loaded layout, if known.
    pub fn current_layout(&self) -> Option<String> {
        self.inner.read().layout.clone()
    }

    /// Weather identifier (0..=2 typically).
    pub fn weather(&self) -> Option<u8> {
        self.inner.read().weather
    }

    /// Wind conditions.
    pub fn wind(&self) -> Option<Wind> {
        self.inner.read().wind
    }

    /// Current session state.
    pub fn session(&self) -> SessionState {
        self.inner.read().session.clone()
    }

    /// Overall game flags.
    pub fn flags(&self) -> StaFlags {
        self.inner.read().flags
    }

    /// Current multiplayer state.
    pub fn multiplayer(&self) -> MultiplayerState {
        self.inner.read().multiplayer.clone()
    }

    /// Server's allowed-cars set, if a `Small`/`Alc` has been received.
    pub fn allowed_cars(&self) -> Option<PlcAllowedCarsSet> {
        self.inner.read().allowed_cars.clone()
    }

    /// Server's allowed-mods list, if a `Mal` has been received.
    pub fn allowed_mods(&self) -> Vec<Vehicle> {
        self.inner.read().allowed_mods.clone()
    }

    /// Version information about the connected LFS instance.
    pub fn version(&self) -> Option<VersionInfo> {
        self.inner.read().version.clone()
    }

    /// Snapshot of the current game state as a [`GameInfo`].
    pub fn game_info(&self) -> GameInfo {
        let g = self.inner.read();
        GameInfo {
            track: g.track,
            layout: g.layout.clone(),
            weather: g.weather,
            wind: g.wind,
            session: g.session.clone(),
            flags: g.flags,
            multiplayer: g.multiplayer.clone(),
            allowed_cars: g.allowed_cars.clone(),
            allowed_mods: g.allowed_mods.clone(),
            version: g.version.clone(),
            axi_count: g.axi_count,
            rst_count: g.rst_count,
        }
    }

    // ── Async wait-for (game) ─────────────────────────────────────────────────

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
    /// session is no longer [`SessionState::Unknown`] and the current track
    /// is known.
    pub async fn wait_for_known_state(&self, cancel: CancellationToken) -> Option<()> {
        self.wait_for_game(
            |info| {
                !matches!(info.session(), &SessionState::Unknown) && info.current_track().is_some()
            },
            Duration::from_millis(100),
            cancel,
        )
        .await
    }

    /// Wait until the game is no longer in progress.
    pub async fn wait_for_end(&self, cancel: CancellationToken) -> Option<()> {
        self.wait_for_game(
            |info| matches!(info.session(), &SessionState::Lobby),
            Duration::from_millis(500),
            cancel,
        )
        .await
    }

    /// Wait until the given track is loaded and the session is
    /// [`SessionState::Lobby`] (selection screen, not yet racing).
    pub async fn wait_for_track(&self, track: Track, cancel: CancellationToken) -> Option<()> {
        self.wait_for_game(
            move |info| {
                info.current_track() == Some(&track)
                    && matches!(info.session(), &SessionState::Lobby)
            },
            Duration::from_millis(500),
            cancel,
        )
        .await
    }

    /// Wait until a race session is in progress.
    pub async fn wait_for_racing(&self, cancel: CancellationToken) -> Option<()> {
        self.wait_for_game(
            |info| matches!(info.session(), &SessionState::Racing { .. }),
            Duration::from_millis(500),
            cancel,
        )
        .await
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
        let before = self.inner.read().axi_count;
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
        let before = self.inner.read().rst_count;
        self.wait_for_game(
            move |info| info.rst_count != before,
            Duration::from_millis(100),
            cancel,
        )
        .await
    }

    // ── Game command methods ──────────────────────────────────────────────────

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

    // ── Presence command methods ──────────────────────────────────────────────

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

    // ── Race accessor ─────────────────────────────────────────────────────────

    /// Access the underlying [`RaceTracker`].
    pub fn race(&self) -> &RaceTracker {
        &self.race
    }
}
