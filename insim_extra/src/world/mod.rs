//! [`World`] is the unified state mirror combining connection/player tracking,
//! game-state mirroring, and race tracking in a single structure.
//!
//! Feed packets with [`apply_packet`](World::apply_packet) to update state and
//! collect change events, then send whatever it asks for with
//! [`take_requests`](World::take_requests). Or let
//! [`SharedWorld::attach`] keep one in step with a split session. Query
//! connection/player/game state with borrowing getters, and issue admin
//! commands that return [`insim::Packet`] values for the caller to send.
//!
//! Replies to requests are told apart from live changes: a connection or player
//! that was already there arrives with `existing: true`, and a reply that only
//! repeats what the world knows produces no event at all.
//!
//! ```ignore
//! let mut world = World::new();
//!
//! loop {
//!     let packet = session.read().await?;
//!     for event in world.apply_packet(&packet) {
//!         match event {
//!             WorldEvent::Connected(Connected { info, existing: false }) => { /* greet */ }
//!             WorldEvent::SessionStarted(SessionStarted { kind }) => { /* ... */ }
//!             WorldEvent::Race(re) => { /* ... */ }
//!             _ => {}
//!         }
//!     }
//!     for request in world.take_requests() {
//!         session.write(request).await?;
//!     }
//! }
//! ```

use std::{collections::HashMap, sync::Arc, time::Duration};

use insim::{
    WithRequestId,
    core::{track::Track, vehicle::Vehicle, wind::Wind},
    identifiers::{ConnectionId, PlayerId, RequestId},
    insim::{
        Axi, Cnl, Cpr, Ism, Nci, Ncn, Npl, Pfl, Pla, PlcAllowedCarsSet, Pll, RaceFlags,
        RaceInProgress, RaceLaps, Rst, Slc, SmallType, Sta, StaFlags, Tiny, TinyType, Toc, Ver,
    },
};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

mod attach;
mod commands;
mod connection;
mod event;
mod game;
mod race;

use connection::MultiIndexPlayerInfoMap;
pub use connection::{ConnectionInfo, PlayerInfo};
pub use event::{
    AllowedCarsChanged, AllowedModsChanged, Connected, ConnectionDetails, Disconnected,
    LayoutChanged, LobbyEntered, LobbyNotReady, LobbyReady, MultiplayerJoined, MultiplayerLeft,
    PlayerJoined, PlayerLeft, PlayerTeleportedToPits, Renamed, SessionEnded, SessionStarted,
    TakingOver, TrackChanged, VehicleSelected, VersionReceived, WorldEvent, WorldReady,
};
pub use game::{
    GameInfo, GridMode, Month, MultiplayerState, SessionKind, TimeDemoPreset, TimeSet, VersionInfo,
};
use race::RaceState;
pub use race::{
    DriverRecord, EntrantId, EntrantState, FinishStatus, LapRecord, PitRecord, RaceEvent,
};

/// Unified state mirror combining connection/player tracking, game-state
/// mirroring, and race tracking.
///
/// Fold packets in with [`apply_packet`](Self::apply_packet) and read it with
/// borrowing getters. kitcar hands services a `&World` for each input, settled as
/// of that input. To share one between tasks, wrap it in a [`SharedWorld`].
///
/// Use [`World::new()`] for short-form races and qualifying, where every player
/// join is a fresh entrant. Use [`World::with_rejoin()`] for long-form races
/// (endurance / multi-hour) where a player may briefly disconnect and reconnect.
///
/// Multiplayer only: the local connection (UCID 0) is the server, never a player,
/// so [`connections`](Self::connections) leaves it out.
#[derive(Default)]
pub struct World {
    // Presence:
    pub(crate) connections: HashMap<ConnectionId, ConnectionInfo>,
    players: MultiIndexPlayerInfoMap,
    /// Survives `Cnl`: maps LFS.net username to last seen display name.
    last_known_names: HashMap<String, String>,

    /// Game state snapshot, including the current `session_kind`.
    pub(crate) game: GameInfo,
    /// Whether at least one STA game-state snapshot has been received.
    /// Until then, `session_kind == None` means unknown rather than lobby.
    sta_received: bool,

    /// Every reply to the startup requests has been folded.
    ready: bool,

    /// Requests the world needs sent; see [`World::take_requests`].
    requests: Vec<insim::Packet>,

    /// Race tracking state.
    pub(crate) race: RaceState,

    /// When `true`, a joining player first attempts to resume a prior
    /// disconnected entrant (matched by LFS.net username) before falling back
    /// to a fresh entrant. Set once at construction.
    pub(crate) rejoin: bool,
}

fn lobby_ready(inner: &World) -> bool {
    inner.sta_received
        && inner.game.session_kind.is_none()
        && inner.connections.keys().any(|ucid| ucid.0 != 0)
}

impl World {
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

    fn apply_tiny_clr(&mut self, tiny: &Tiny) -> Vec<PlayerInfo> {
        if matches!(tiny.subt, TinyType::Clr) {
            let players = self
                .players
                .iter()
                .map(|(_, player)| player.clone())
                .collect();
            self.players.clear();
            players
        } else {
            Vec::new()
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
fn dispatch(inner: &mut World, packet: &insim::Packet, events: &mut Vec<WorldEvent>) {
    use insim::Packet;

    // Wrap a race-handler call, pushing each emitted [`RaceEvent`]. The handlers
    // live on `World` and read `session_kind`/`connections` themselves.
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
            // A reply describes a connection that was already there. If we know
            // it already, it changes nothing worth reporting.
            let existing = ncn.reqi.0 != 0;
            let known = inner.connections.contains_key(&ncn.ucid);
            let info = inner.apply_ncn(ncn);
            if !(existing && known) {
                events.push(WorldEvent::Connected(Connected { info, existing }));
            }
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
            let existing = npl.reqi.0 != 0;
            let known = inner.players.get_by_plid(&npl.plid).is_some();
            if let Some(info) = inner.apply_npl(npl) {
                // The race layer always sees the join: after a session restart it
                // recreates entrants from these re-announcements.
                push_race!(inner.on_player_joined(&info));
                if !(existing && known) {
                    events.push(WorldEvent::PlayerJoined(PlayerJoined { info, existing }));
                }
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
            for player in inner.apply_tiny_clr(tiny) {
                push_race!(inner.on_player_left(&player));
                events.push(WorldEvent::PlayerLeft(PlayerLeft(player)));
            }
            if let Some(prev) = inner.apply_tiny_axc(tiny) {
                events.push(WorldEvent::LayoutChanged(LayoutChanged {
                    from: Some(prev),
                    to: None,
                }));
            }
        },
        Packet::Sta(sta) => {
            let (was_in_session, now_in_session, prev_track, new_track) = inner.apply_sta(sta);
            inner.sta_received = true;
            if was_in_session && !now_in_session {
                events.push(WorldEvent::SessionEnded(SessionEnded));
                events.push(WorldEvent::LobbyEntered(LobbyEntered));
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
                // A new session can change the grid, allowed cars and mods.
                inner.requests.extend(
                    World::SESSION_REQUESTS
                        .iter()
                        .map(|t| t.clone().with_request_id(RequestId(1)).into()),
                );
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

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("connections", &self.connections.len())
            .field("players", &self.players.len())
            .field("track", &self.game.track)
            .field("session_kind", &self.game.session_kind)
            .field("sta_received", &self.sta_received)
            .field("ready", &self.ready)
            .field("race_entrants", &self.race.entrants.len())
            .field("rejoin", &self.rejoin)
            .finish_non_exhaustive()
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
            ..Default::default()
        }
    }

    /// Apply one raw packet, update all state, and return any events.
    ///
    /// A single match over the packet type drives the connection/player, game,
    /// and race mirrors in one pass. Replies to requests (a nonzero ReqI) that
    /// only repeat what the world already knows produce no events; those that
    /// introduce something are marked `existing`. Afterwards,
    /// [`take_requests`](Self::take_requests) has anything the world needs sent.
    pub fn apply_packet(&mut self, packet: &insim::Packet) -> Vec<WorldEvent> {
        let mut events = Vec::new();
        let was_lobby_ready = lobby_ready(self);
        dispatch(self, packet, &mut events);
        match (was_lobby_ready, lobby_ready(self)) {
            (false, true) => events.push(WorldEvent::LobbyReady(LobbyReady)),
            (true, false) => events.push(WorldEvent::LobbyNotReady(LobbyNotReady)),
            _ => {},
        }
        events
    }

    /// Requests the world needs sent to stay in step, such as
    /// [`SESSION_REQUESTS`](Self::SESSION_REQUESTS) after a session starts.
    /// Whoever folds packets in sends these.
    pub fn take_requests(&mut self) -> Vec<insim::Packet> {
        std::mem::take(&mut self.requests)
    }

    /// Whether the world has caught up with the server: every reply to
    /// [`STARTUP_REQUESTS`](Self::STARTUP_REQUESTS) has been folded, so it
    /// mirrors the connections, players and track that existed before you
    /// connected. Something started after this point never sees
    /// [`WorldEvent::Ready`], so it should check this instead.
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    /// Record that the world has caught up. Called by whatever keeps it in step
    /// ([`SharedWorld::attach`], or kitcar's host) just before it reports
    /// [`WorldEvent::Ready`]. Returns `true` the first time.
    pub fn mark_ready(&mut self) -> bool {
        !std::mem::replace(&mut self.ready, true)
    }

    /// Players' connections, leaving out the server's own (UCID 0).
    pub fn connections(&self) -> impl Iterator<Item = &ConnectionInfo> {
        self.connections.values().filter(|c| !c.ucid.local())
    }

    /// Number of players' connections, leaving out the server's own.
    pub fn connection_count(&self) -> usize {
        self.connections().count()
    }

    /// Look up one connection by UCID, including the server's own.
    pub fn connection(&self, ucid: ConnectionId) -> Option<&ConnectionInfo> {
        self.connections.get(&ucid)
    }

    /// All tracked players.
    pub fn players(&self) -> impl Iterator<Item = &PlayerInfo> {
        self.players.iter().map(|(_, p)| p)
    }

    /// Number of tracked players.
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Look up one player by `PlayerId`.
    pub fn player(&self, plid: PlayerId) -> Option<&PlayerInfo> {
        self.players.get_by_plid(&plid)
    }

    /// The players a connection controls.
    pub fn players_of(&self, ucid: ConnectionId) -> impl Iterator<Item = &PlayerInfo> {
        self.players().filter(move |p| p.ucid == ucid)
    }

    /// Look up the connection that currently controls a given player.
    pub fn connection_by_player(&self, plid: PlayerId) -> Option<&ConnectionInfo> {
        self.connections.get(&self.player(plid)?.ucid)
    }

    /// Last known display name for an LFS.net username. Survives disconnect.
    pub fn last_known_name(&self, uname: &str) -> Option<&str> {
        self.last_known_names.get(uname).map(String::as_str)
    }

    /// Currently selected track, if known.
    pub fn track(&self) -> Option<Track> {
        self.game.track
    }

    /// Currently loaded layout, if known.
    pub fn layout(&self) -> Option<&str> {
        self.game.layout.as_deref()
    }

    /// Weather identifier (0..=2 typically).
    pub fn weather(&self) -> Option<u8> {
        self.game.weather
    }

    /// Wind conditions.
    pub fn wind(&self) -> Option<Wind> {
        self.game.wind
    }

    /// Current session kind. `None` means no active session, or that the
    /// initial `STA` game-state snapshot has not arrived yet.
    pub fn session(&self) -> Option<SessionKind> {
        self.game.session_kind
    }

    /// Overall game flags.
    pub fn flags(&self) -> StaFlags {
        self.game.flags
    }

    /// Current multiplayer state.
    pub fn multiplayer(&self) -> &MultiplayerState {
        &self.game.multiplayer
    }

    /// Server's allowed-cars set, if a `Small`/`Alc` has been received.
    pub fn allowed_cars(&self) -> Option<&PlcAllowedCarsSet> {
        self.game.allowed_cars.as_ref()
    }

    /// Server's allowed-mods list, if a `Mal` has been received.
    pub fn allowed_mods(&self) -> &[Vehicle] {
        &self.game.allowed_mods
    }

    /// Version information about the connected LFS instance.
    pub fn version(&self) -> Option<&VersionInfo> {
        self.game.version.as_ref()
    }

    /// The current game state.
    pub fn game_info(&self) -> &GameInfo {
        &self.game
    }

    /// The current session fastest lap: entrant, player ID, and time.
    pub fn fastest_lap(&self) -> Option<(EntrantId, PlayerId, Duration)> {
        self.race.fastest_lap
    }

    /// Whether a race session (not qualifying or practice) is currently active.
    pub fn race_active(&self) -> bool {
        self.game.session_kind.is_some_and(|k| k.is_race())
    }

    /// Look up an entrant by their stable [`EntrantId`].
    pub fn entrant(&self, id: EntrantId) -> Option<&EntrantState> {
        self.race.entrants.get(&id)
    }

    /// Look up the currently-live entrant for a [`PlayerId`].
    pub fn entrant_by_plid(&self, plid: PlayerId) -> Option<&EntrantState> {
        self.race.entrants.get(self.race.live.get(&plid)?)
    }

    /// All entrants (racing, finished, and DNF).
    pub fn entrants(&self) -> impl Iterator<Item = &EntrantState> {
        self.race.entrants.values()
    }

    /// Entrants currently on track.
    pub fn live_entrants(&self) -> impl Iterator<Item = &EntrantState> {
        self.race
            .live
            .values()
            .filter_map(|id| self.race.entrants.get(id))
    }
}

/// A [`World`] shared between tasks: a cheap, cloneable handle. Use it where
/// several places read one world, such as [`SharedWorld::attach`] or kitcar's
/// `Host::world()`. Lock it briefly with [`read`](Self::read); never hold the
/// guard across an `.await`.
#[derive(Clone, Default)]
pub struct SharedWorld {
    inner: Arc<RwLock<World>>,
}

impl SharedWorld {
    /// Share `world`.
    pub fn new(world: World) -> Self {
        Self {
            inner: Arc::new(RwLock::new(world)),
        }
    }
    /// Read the world.
    pub fn read(&self) -> RwLockReadGuard<'_, World> {
        self.inner.read()
    }
    /// Change the world, as whoever folds packets into it does.
    pub fn write(&self) -> RwLockWriteGuard<'_, World> {
        self.inner.write()
    }
}

impl std::fmt::Debug for SharedWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.read().fmt(f)
    }
}

#[cfg(test)]
mod emission_tests {
    use insim::{
        core::track::Track,
        identifiers::{ConnectionId, PlayerId},
        insim::{Axi, Cnl, Ncn, Npl, RaceInProgress, RaceLaps, Rst, Sta, Tiny, TinyType},
    };

    use super::{World, WorldEvent};

    fn count(events: &[WorldEvent], pred: impl Fn(&WorldEvent) -> bool) -> usize {
        events.iter().filter(|e| pred(e)).count()
    }

    fn connection(ucid: u8) -> insim::Packet {
        Ncn {
            ucid: ConnectionId(ucid),
            uname: format!("user-{ucid}"),
            pname: format!("driver-{ucid}"),
            ..Default::default()
        }
        .into()
    }

    #[test]
    fn rst_emits_session_started() {
        let mut world = World::new();
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
    fn tiny_clr_reconciles_players_and_live_race_entrants() {
        let mut world = World::new();
        let _ = world.apply_packet(
            &Ncn {
                ucid: ConnectionId(1),
                uname: "user".into(),
                pname: "driver".into(),
                ..Default::default()
            }
            .into(),
        );
        let _ = world.apply_packet(
            &Rst {
                racelaps: RaceLaps::Laps(5),
                ..Default::default()
            }
            .into(),
        );
        let _ = world.apply_packet(
            &Npl {
                plid: PlayerId(1),
                ucid: ConnectionId(1),
                nump: 1,
                pname: "driver".into(),
                ..Default::default()
            }
            .into(),
        );

        let events = world.apply_packet(
            &Tiny {
                subt: TinyType::Clr,
                ..Default::default()
            }
            .into(),
        );

        assert_eq!(world.player_count(), 0);
        assert!(world.live_entrants().next().is_none());
        assert!(
            events
                .iter()
                .any(|event| matches!(event, WorldEvent::PlayerLeft(_)))
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, WorldEvent::Race(super::RaceEvent::Dnf { .. })))
        );
    }

    #[test]
    fn sta_emits_session_ended_on_leaving_race() {
        let mut world = World::new();

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
        assert_eq!(
            count(&events, |e| matches!(e, WorldEvent::LobbyEntered(_))),
            1,
            "LobbyEntered should fire once on Racing -> No transition"
        );
    }

    #[test]
    fn lobby_becomes_ready_when_entered_with_a_client_present() {
        let mut world = World::new();
        let _ = world.apply_packet(&connection(0));
        let _ = world.apply_packet(&connection(1));
        let _ = world.apply_packet(
            &Rst {
                racelaps: RaceLaps::Laps(5),
                ..Default::default()
            }
            .into(),
        );

        let events = world.apply_packet(
            &Sta {
                raceinprog: RaceInProgress::No,
                ..Default::default()
            }
            .into(),
        );

        assert_eq!(world.connection_count(), 1);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WorldEvent::LobbyEntered(_)))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WorldEvent::LobbyReady(_)))
        );
    }

    #[test]
    fn lobby_becomes_ready_when_a_client_joins_in_the_lobby() {
        let mut world = World::new();
        let server_events = world.apply_packet(&connection(0));
        assert_eq!(world.connection_count(), 0);
        assert!(
            !server_events
                .iter()
                .any(|e| matches!(e, WorldEvent::LobbyReady(_)))
        );

        let sta_events = world.apply_packet(
            &Sta {
                raceinprog: RaceInProgress::No,
                ..Default::default()
            }
            .into(),
        );
        assert!(
            !sta_events
                .iter()
                .any(|e| matches!(e, WorldEvent::LobbyReady(_)))
        );

        let events = world.apply_packet(&connection(1));
        assert_eq!(world.connection_count(), 1);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WorldEvent::LobbyReady(_)))
        );

        let events = world.apply_packet(
            &Cnl {
                ucid: ConnectionId(1),
                ..Default::default()
            }
            .into(),
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WorldEvent::LobbyNotReady(_)))
        );
    }

    #[test]
    fn client_before_initial_sta_does_not_imply_lobby_ready() {
        let mut world = World::new();

        let events = world.apply_packet(&connection(1));
        assert_eq!(world.connection_count(), 1);
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, WorldEvent::LobbyReady(_)))
        );

        let events = world.apply_packet(
            &Sta {
                raceinprog: RaceInProgress::No,
                ..Default::default()
            }
            .into(),
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WorldEvent::LobbyReady(_)))
        );
    }

    #[test]
    fn restart_does_not_emit_lobby_entered() {
        let mut world = World::new();
        let rst = || {
            insim::Packet::Rst(Rst {
                racelaps: RaceLaps::Laps(5),
                ..Default::default()
            })
        };
        let _ = world.apply_packet(&rst());

        let events = world.apply_packet(&rst());
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WorldEvent::SessionEnded(_)))
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, WorldEvent::LobbyEntered(_)))
        );
    }

    #[test]
    fn sta_emits_track_changed_on_track_field_change() {
        let mut world = World::new();
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
        let mut world = World::new();
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

#[cfg(test)]
mod reply_tests {
    use insim::{
        Packet,
        identifiers::{ConnectionId, PlayerId, RequestId},
        insim::{Ncn, Npl, Rst, TinyType},
    };

    use super::{Connected, PlayerJoined, World, WorldEvent};

    fn ncn(reqi: u8, ucid: u8) -> Packet {
        Ncn {
            reqi: RequestId(reqi),
            ucid: ConnectionId(ucid),
            uname: format!("user{ucid}"),
            ..Default::default()
        }
        .into()
    }
    fn npl(reqi: u8, plid: u8) -> Packet {
        Npl {
            reqi: RequestId(reqi),
            ucid: ConnectionId(1),
            plid: PlayerId(plid),
            nump: 1,
            ..Default::default()
        }
        .into()
    }
    fn connected(events: &[WorldEvent]) -> Vec<bool> {
        events
            .iter()
            .filter_map(|e| match e {
                WorldEvent::Connected(Connected { existing, .. }) => Some(*existing),
                _ => None,
            })
            .collect()
    }
    fn joined(events: &[WorldEvent]) -> Vec<bool> {
        events
            .iter()
            .filter_map(|e| match e {
                WorldEvent::PlayerJoined(PlayerJoined { existing, .. }) => Some(*existing),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn replies_mark_what_was_already_there_and_repeat_nothing() {
        let mut world = World::new();
        // A startup reply introduces someone who was already connected.
        assert_eq!(connected(&world.apply_packet(&ncn(130, 1))), [true]);
        // The same reply again (e.g. another request) reports nothing.
        assert!(connected(&world.apply_packet(&ncn(131, 1))).is_empty());
        // Someone joining now is not existing.
        assert_eq!(connected(&world.apply_packet(&ncn(0, 2))), [false]);

        assert_eq!(joined(&world.apply_packet(&npl(130, 7))), [true]);
        // A re-announcement in reply to a session request is silent...
        assert!(joined(&world.apply_packet(&npl(1, 7))).is_empty());
        // ...but a live rejoin is reported.
        assert_eq!(joined(&world.apply_packet(&npl(0, 7))), [false]);
        assert_eq!(world.connection_count(), 2);
    }

    #[test]
    fn a_new_session_asks_for_the_session_requests() {
        let mut world = World::new();
        assert!(world.take_requests().is_empty());
        let _ = world.apply_packet(&Rst::default().into());
        let requested: Vec<TinyType> = world
            .take_requests()
            .into_iter()
            .filter_map(|p| match p {
                Packet::Tiny(tiny) => Some(tiny.subt),
                _ => None,
            })
            .collect();
        assert_eq!(requested, World::SESSION_REQUESTS);
        assert!(world.take_requests().is_empty(), "taken once");
    }
}
