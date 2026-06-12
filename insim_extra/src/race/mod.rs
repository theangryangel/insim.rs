//! [`RaceTracker`] accumulates lap, split, pit-stop and result data.
//!
//! Call the appropriate `apply_*` method as events and packets arrive. Methods
//! that can produce state-change notifications return [`Vec<RaceEvent>`];
//! methods that only update internal bookkeeping return nothing.
//!
//! The convenience methods [`RaceTracker::apply_presence_event`] and
//! [`RaceTracker::apply_game_event`] route entire event enums in one call:
//!
//! ```ignore
//! while let Some(packet) = conn.next().await {
//!     for event in presence.apply_packet(&packet) {
//!         for e in race.apply_presence_event(&event) { /* handle */ }
//!     }
//!     for event in game.apply_packet(&packet) {
//!         for e in race.apply_game_event(&event) { /* handle */ }
//!     }
//!     let events = match &packet {
//!         insim::Packet::Lap(v) => race.apply_lap(v),
//!         insim::Packet::Spx(v) => race.apply_split(v),
//!         insim::Packet::Fin(v) => race.apply_finish(v),
//!         insim::Packet::Res(v) => race.apply_result(v),
//!         insim::Packet::Pit(v) => race.apply_pit_stop(v),
//!         insim::Packet::Psf(v) => race.apply_pit_stop_finished(v),
//!         insim::Packet::Pen(v) => race.apply_penalty_changed(v),
//!         _ => vec![],
//!     };
//! }
//! ```

mod entrant;
mod event;

use std::{collections::HashMap, sync::Arc, time::Duration};

pub use entrant::{DriverRecord, EntrantId, EntrantState, FinishStatus, LapRecord, PitRecord};
pub use event::RaceEvent;
use insim::{
    Packet,
    identifiers::{ConnectionId, PlayerId},
    insim::{Fin, Lap, Pen, PenaltyInfo, Pit, Plp, Psf, Reo, Res, Spx, TinyType},
};
use parking_lot::RwLock;

use crate::{
    game::{GameEvent, SessionKind},
    presence::{ConnectionInfo, PlayerInfo, PresenceEvent},
};

/// LFS uses 1:00:00.000 as a placeholder for an invalid/missing split time.
const INVALID_SPLIT: Duration = Duration::from_secs(3600);

#[derive(Default)]
struct RaceTrackerInner {
    next_id: u64,
    entrants: HashMap<EntrantId, EntrantState>,
    /// Maps currently-active [`PlayerId`] -> [`EntrantId`]. Removed on player
    /// left; the entrant record in `entrants` is kept for post-race querying.
    live: HashMap<PlayerId, EntrantId>,
    /// Name cache: populated from connection joined/renamed events so that
    /// `uname` can be baked into [`DriverRecord`] at join/swap time.
    connections: HashMap<ConnectionId, (String, String)>,
    race_active: bool,
    /// Kind of the current session, set on [`RaceTracker::apply_session_started`].
    /// Used to interpret `Fin` correctly (a finish in a race, a per-lap signal
    /// in qualifying).
    session_kind: Option<SessionKind>,
    /// Grid positions from a `Reo` that arrived before the matching `Npl`.
    /// Drained into [`EntrantState::grid_position`] when the player joins.
    pending_grid: HashMap<PlayerId, u8>,
    /// Session fastest lap: set the first time any entrant completes a lap,
    /// updated whenever a faster time is recorded.
    fastest_lap: Option<(EntrantId, PlayerId, Duration)>,
}

impl RaceTrackerInner {
    fn alloc_id(&mut self) -> EntrantId {
        let id = EntrantId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    fn uname_for(&self, ucid: ConnectionId) -> Option<String> {
        self.connections.get(&ucid).map(|(uname, _)| uname.clone())
    }

    fn pname_for(&self, ucid: ConnectionId) -> Option<String> {
        self.connections.get(&ucid).map(|(_, pname)| pname.clone())
    }
}

/// Accumulates race data from insim packets and presence/game events.
///
/// State lives behind `Arc<RwLock<…>>`; clones are cheap and share the same
/// maps.
#[derive(Clone, Default)]
pub struct RaceTracker {
    inner: Arc<RwLock<RaceTrackerInner>>,
}

impl std::fmt::Debug for RaceTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let g = self.inner.read();
        f.debug_struct("RaceTracker")
            .field("entrants", &g.entrants.len())
            .field("live", &g.live.len())
            .field("race_active", &g.race_active)
            .finish()
    }
}

impl RaceTracker {
    /// Tiny requests to re-send on each [`RaceEvent::SessionStarted`] so the
    /// cleared entrant list is rebuilt (`Npl`) and the starting grid order is
    /// fetched (`Reo`). Requesting `Reo` in a practice/untimed session is
    /// harmless - LFS replies with no grid and it is ignored.
    pub const SESSION_REQUESTS: &[TinyType] = &[TinyType::Npl, TinyType::Reo];

    /// Create a new tracker with empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all state. The `next_id` counter is preserved so previously
    /// issued [`EntrantId`] values are never reused.
    pub fn clear(&self) {
        let mut g = self.inner.write();
        g.entrants.clear();
        g.live.clear();
        g.connections.clear();
        g.pending_grid.clear();
        g.race_active = false;
        g.session_kind = None;
        g.fastest_lap = None;
    }

    /// The kind of the current session, if one has started.
    pub fn session_kind(&self) -> Option<SessionKind> {
        self.inner.read().session_kind
    }

    /// The current session fastest lap: the entrant, their player ID, and the
    /// time. `None` until at least one lap has been completed.
    pub fn fastest_lap(&self) -> Option<(EntrantId, PlayerId, Duration)> {
        self.inner.read().fastest_lap
    }

    /// Whether a race session is currently active.
    pub fn race_active(&self) -> bool {
        self.inner.read().race_active
    }

    /// Look up an entrant by their stable [`EntrantId`].
    pub fn entrant(&self, id: EntrantId) -> Option<EntrantState> {
        self.inner.read().entrants.get(&id).cloned()
    }

    /// Look up the currently-live entrant for a [`PlayerId`].
    pub fn entrant_by_plid(&self, plid: PlayerId) -> Option<EntrantState> {
        let g = self.inner.read();
        let id = g.live.get(&plid)?;
        g.entrants.get(id).cloned()
    }

    /// Snapshot of all entrants (racing, finished, and DNF).
    pub fn entrants(&self) -> Vec<EntrantState> {
        self.inner.read().entrants.values().cloned().collect()
    }

    /// Snapshot of entrants who are currently on track.
    pub fn live_entrants(&self) -> Vec<EntrantState> {
        let g = self.inner.read();
        g.live
            .values()
            .filter_map(|id| g.entrants.get(id).cloned())
            .collect()
    }

    // ── Connection name cache ─────────────────────────────────────────────────

    /// Record a connection's names. Call on `Connected` / `PresenceEvent::Connected`.
    pub fn apply_connected(&self, info: &ConnectionInfo) {
        let mut g = self.inner.write();
        let _ = g
            .connections
            .insert(info.ucid, (info.uname.clone(), info.pname.clone()));
    }

    /// Evict a connection from the name cache. Call on `Disconnected` / `PresenceEvent::Disconnected`.
    pub fn apply_disconnected(&self, ucid: ConnectionId) {
        let _ = self.inner.write().connections.remove(&ucid);
    }

    /// Update a connection's display name. Call on `Renamed` / `PresenceEvent::Renamed`.
    pub fn apply_renamed(&self, ucid: ConnectionId, uname: &str, pname: &str) {
        let mut g = self.inner.write();
        let _ = g
            .connections
            .insert(ucid, (uname.to_owned(), pname.to_owned()));
    }

    /// Register a fresh entrant with a new [`EntrantId`] and zeroed lap state.
    ///
    /// **Use this in short-form races** (sprints, qualifying) for every join,
    /// and in long-form races for genuinely new entrants who have no prior
    /// history to restore.
    ///
    /// If you are running a long-form race and a player may reconnect after a
    /// brief disconnect, call [`apply_player_rejoined`](Self::apply_player_rejoined)
    /// instead - it will attempt to resume the prior [`EntrantState`] and falls
    /// back to this behaviour automatically if no match is found.
    pub fn apply_player_joined(&self, info: &PlayerInfo) -> Vec<RaceEvent> {
        let mut g = self.inner.write();
        // Idempotent per live plid: LFS announces each player with an `Npl` at
        // grid formation *and* again in reply to a `TINY_NPL` request, so the
        // same plid can arrive twice. If it is already live, fold in any pending
        // grid position and ignore the duplicate rather than creating a phantom
        // second entrant.
        if let Some(&id) = g.live.get(&info.plid) {
            if let Some(grid) = g.pending_grid.remove(&info.plid)
                && let Some(entrant) = g.entrants.get_mut(&id)
            {
                entrant.grid_position = Some(grid);
            }
            return vec![];
        }
        let id = g.alloc_id();
        let uname = g.uname_for(info.ucid);
        let pname = g.pname_for(info.ucid).unwrap_or_else(|| info.pname.clone());
        let grid_position = g.pending_grid.remove(&info.plid);
        let state = EntrantState {
            id,
            plid: info.plid,
            laps_done: 0,
            lap_offset: 0,
            best_lap: None,
            best_lap_num: None,
            laps: Vec::new(),
            current_splits: Vec::new(),
            pit_stops: Vec::new(),
            status: FinishStatus::Racing,
            drivers: vec![DriverRecord {
                ucid: info.ucid,
                pname,
                uname,
                from_lap: 0,
            }],
            grid_position,
            pending_pit: None,
            penalty: PenaltyInfo::None,
        };
        let _ = g.entrants.insert(id, state);
        let _ = g.live.insert(info.plid, id);
        vec![RaceEvent::EntrantJoined {
            id,
            plid: info.plid,
        }]
    }

    /// Mark an entrant as DNF if the race is active. Call on `PlayerLeft` / `PresenceEvent::PlayerLeft`.
    pub fn apply_player_left(&self, info: &PlayerInfo) -> Vec<RaceEvent> {
        let mut g = self.inner.write();
        let Some(id) = g.live.remove(&info.plid) else {
            return vec![];
        };
        if g.race_active
            && let Some(entrant) = g.entrants.get_mut(&id)
            && matches!(entrant.status, FinishStatus::Racing)
        {
            entrant.status = FinishStatus::Dnf;
            return vec![RaceEvent::Dnf {
                id,
                plid: info.plid,
            }];
        }
        vec![]
    }

    /// Record a driver swap. Call on `TakingOver` / `PresenceEvent::TakingOver`.
    pub fn apply_taking_over(&self, before: &PlayerInfo, after: &PlayerInfo) -> Vec<RaceEvent> {
        let mut g = self.inner.write();
        let Some(&id) = g.live.get(&before.plid) else {
            return vec![];
        };
        let uname = g.uname_for(after.ucid);
        let pname = g
            .pname_for(after.ucid)
            .unwrap_or_else(|| after.pname.clone());
        if let Some(entrant) = g.entrants.get_mut(&id) {
            entrant.drivers.push(DriverRecord {
                ucid: after.ucid,
                pname,
                uname,
                from_lap: entrant.laps_done,
            });
        }
        vec![RaceEvent::DriverSwap {
            id,
            plid: before.plid,
            new_ucid: after.ucid,
        }]
    }

    /// Attempt to resume a disconnected entrant, falling back to a fresh join.
    ///
    /// **Use this in long-form races** (30+ minutes, multi-hour, endurance)
    /// where a player may briefly lose connection and reconnect mid-race.
    ///
    /// **Matching:** searches for a disconnected entrant (present in `entrants`
    /// but no longer `live`) whose last [`DriverRecord`] has a `uname` matching
    /// the incoming connection's LFS.net username from the connection cache.
    ///
    /// - If a match is found, the existing [`EntrantId`] is reused, all prior
    ///   lap/split/pit history is preserved, a new [`DriverRecord`] is appended,
    ///   and a [`lap_offset`](EntrantState::lap_offset) is set so subsequent
    ///   [`apply_lap`](Self::apply_lap) calls correct LFS's reset-to-1 counter
    ///   to the true running total. Returns [`RaceEvent::EntrantRejoined`].
    ///
    /// - If the incoming connection has no `uname` (guest / unregistered), or
    ///   no disconnected entrant matches, a fresh entrant is created instead and
    ///   [`RaceEvent::EntrantJoined`] is returned.
    ///
    /// **Do not use for short-form races.** In a sprint or qualifying session
    /// every join should be treated as a new entrant; using this method risks
    /// accidentally merging unrelated entries.
    pub fn apply_player_rejoined(&self, info: &PlayerInfo) -> Vec<RaceEvent> {
        let mut g = self.inner.write();
        // Idempotent per live plid: ignore a duplicate `Npl` for an entrant that
        // is already on track (see [`apply_player_joined`](Self::apply_player_joined)).
        if let Some(&id) = g.live.get(&info.plid) {
            if let Some(grid) = g.pending_grid.remove(&info.plid)
                && let Some(entrant) = g.entrants.get_mut(&id)
            {
                entrant.grid_position = Some(grid);
            }
            return vec![];
        }
        let uname = g.uname_for(info.ucid);
        if let Some(prior_id) = uname
            .as_deref()
            .and_then(|u| find_disconnected_by_uname(&g, u))
        {
            let pname = g.pname_for(info.ucid).unwrap_or_else(|| info.pname.clone());
            let entrant = g.entrants.get_mut(&prior_id).unwrap();
            let pre_offset = entrant.laps_done;
            entrant.lap_offset = entrant.lap_offset.wrapping_add(entrant.laps_done);
            entrant.current_splits.clear();
            entrant.pending_pit = None;
            entrant.plid = info.plid;
            entrant.status = FinishStatus::Racing;
            entrant.drivers.push(DriverRecord {
                ucid: info.ucid,
                pname,
                uname,
                from_lap: pre_offset,
            });
            let _ = g.live.insert(info.plid, prior_id);
            return vec![RaceEvent::EntrantRejoined {
                id: prior_id,
                plid: info.plid,
            }];
        }
        // No match - fresh join
        let id = g.alloc_id();
        let pname = g.pname_for(info.ucid).unwrap_or_else(|| info.pname.clone());
        let grid_position = g.pending_grid.remove(&info.plid);
        let state = EntrantState {
            id,
            plid: info.plid,
            laps_done: 0,
            lap_offset: 0,
            best_lap: None,
            best_lap_num: None,
            laps: Vec::new(),
            current_splits: Vec::new(),
            pit_stops: Vec::new(),
            status: FinishStatus::Racing,
            drivers: vec![DriverRecord {
                ucid: info.ucid,
                pname,
                uname,
                from_lap: 0,
            }],
            grid_position,
            pending_pit: None,
            penalty: PenaltyInfo::None,
        };
        let _ = g.entrants.insert(id, state);
        let _ = g.live.insert(info.plid, id);
        vec![RaceEvent::EntrantJoined {
            id,
            plid: info.plid,
        }]
    }

    /// Clear all per-session state and start a new session of the given
    /// [`SessionKind`]. Call on `SessionStarted` / `GameEvent::SessionStarted`
    /// (driven by the `Rst` packet).
    ///
    /// Clears entrants, the live map, pending grid order, and the fastest lap;
    /// the connection name cache is preserved (connections persist across
    /// sessions). LFS re-sends `Npl` for every player after `Rst`, which
    /// repopulates the entrant list.
    pub fn apply_session_started(&self, kind: SessionKind) -> Vec<RaceEvent> {
        let mut g = self.inner.write();
        g.entrants.clear();
        g.live.clear();
        g.pending_grid.clear();
        g.race_active = true;
        g.session_kind = Some(kind);
        g.fastest_lap = None;
        vec![RaceEvent::SessionStarted { kind }]
    }

    /// Mark the session as no longer active. Call on `SessionEnded` /
    /// `GameEvent::SessionEnded` (LFS returned to the entry/lobby screen).
    ///
    /// Accumulated entrant data is **not** cleared - it stays queryable for
    /// post-session results until the next [`apply_session_started`](Self::apply_session_started).
    pub fn apply_race_ended(&self) {
        self.inner.write().race_active = false;
    }

    /// Route a [`PresenceEvent`] to the appropriate `apply_*` method.
    ///
    /// Handles `Connected`, `Disconnected`, `Renamed`, `PlayerJoined`,
    /// `PlayerLeft`, and `TakingOver`; all other variants are ignored and
    /// return an empty vec.
    pub fn apply_presence_event(&self, event: &PresenceEvent) -> Vec<RaceEvent> {
        match event {
            PresenceEvent::Connected(info) => {
                self.apply_connected(info);
                vec![]
            },
            PresenceEvent::Disconnected { ucid, .. } => {
                self.apply_disconnected(*ucid);
                vec![]
            },
            PresenceEvent::Renamed {
                ucid,
                uname,
                new_pname,
            } => {
                self.apply_renamed(*ucid, uname, new_pname);
                vec![]
            },
            PresenceEvent::PlayerJoined(info) => self.apply_player_joined(info),
            PresenceEvent::PlayerLeft(info) => self.apply_player_left(info),
            PresenceEvent::TakingOver { before, after } => self.apply_taking_over(before, after),
            _ => vec![],
        }
    }

    /// Route a [`GameEvent`] to the appropriate `apply_*` method.
    ///
    /// Handles `SessionStarted` and `SessionEnded`; all other variants are
    /// ignored and return an empty vec.
    pub fn apply_game_event(&self, event: &GameEvent) -> Vec<RaceEvent> {
        match event {
            GameEvent::SessionStarted { kind } => self.apply_session_started(*kind),
            GameEvent::SessionEnded => {
                self.apply_race_ended();
                vec![]
            },
            _ => vec![],
        }
    }

    /// Route a [`Packet`] to the appropriate `apply_*` method.
    pub fn apply_packet(&self, packet: &Packet) -> Vec<RaceEvent> {
        match &packet {
            insim::Packet::Lap(v) => self.apply_lap(v),
            insim::Packet::Spx(v) => self.apply_split(v),
            insim::Packet::Fin(v) => self.apply_finish(v),
            insim::Packet::Res(v) => self.apply_result(v),
            insim::Packet::Pit(v) => self.apply_pit_stop(v),
            insim::Packet::Psf(v) => self.apply_pit_stop_finished(v),
            insim::Packet::Pen(v) => self.apply_penalty_changed(v),
            insim::Packet::Plp(v) => self.apply_telepit(v),
            insim::Packet::Reo(v) => self.apply_grid_order(v),
            _ => vec![],
        }
    }

    /// Record the starting grid order. Call on [`insim::Packet::Reo`].
    ///
    /// Assigns a 1-indexed [`grid_position`](EntrantState::grid_position) to
    /// each entrant in finishing order. `Reo` can arrive before the matching
    /// `Npl` packets (LFS re-sends both after `Rst`); positions for not-yet-seen
    /// players are buffered and applied when they join.
    ///
    /// Returns no events - grid order is queried via the entrant snapshot.
    pub fn apply_grid_order(&self, reo: &Reo) -> Vec<RaceEvent> {
        let mut g = self.inner.write();
        let count = (reo.nump as usize).min(reo.plid.len());
        for (i, &plid) in reo.plid.iter().take(count).enumerate() {
            let pos = (i + 1) as u8;
            match g.live.get(&plid).copied() {
                Some(id) => {
                    if let Some(entrant) = g.entrants.get_mut(&id) {
                        entrant.grid_position = Some(pos);
                    }
                },
                None => {
                    let _ = g.pending_grid.insert(plid, pos);
                },
            }
        }
        vec![]
    }

    /// Record a telepit (Shift+P / `Plp` packet). Call on [`insim::Packet::Plp`].
    ///
    /// The in-progress lap is abandoned, so `current_splits` and any pending
    /// pit stop are discarded. The running lap total
    /// ([`laps_done`](EntrantState::laps_done) /
    /// [`lap_offset`](EntrantState::lap_offset)) is **left untouched**: unlike a
    /// spectate-and-rejoin (which issues a fresh `Npl`), LFS does not reset the
    /// lap counter on a telepit, so neither do we.
    pub fn apply_telepit(&self, plp: &Plp) -> Vec<RaceEvent> {
        let mut g = self.inner.write();
        let Some(&id) = g.live.get(&plp.plid) else {
            return vec![];
        };
        let Some(entrant) = g.entrants.get_mut(&id) else {
            return vec![];
        };
        entrant.current_splits.clear();
        entrant.pending_pit = None;
        vec![RaceEvent::TeleportedToPits { id, plid: plp.plid }]
    }

    /// Record a completed lap. Call on [`insim::Packet::Lap`].
    ///
    /// Ignored once the entrant has finished (a finished car still emits `Lap`
    /// packets during its slow-down lap, which would otherwise corrupt the lap
    /// list and best/fastest-lap tracking).
    pub fn apply_lap(&self, lap: &Lap) -> Vec<RaceEvent> {
        let mut g = self.inner.write();
        let Some(&id) = g.live.get(&lap.plid) else {
            return vec![];
        };
        let Some(entrant) = g.entrants.get_mut(&id) else {
            return vec![];
        };
        if matches!(entrant.status, FinishStatus::Finished { .. }) {
            return vec![];
        }
        let effective_lap = lap.lapsdone.wrapping_add(entrant.lap_offset);
        let record = LapRecord {
            lap: effective_lap,
            time: lap.ltime,
            splits: std::mem::take(&mut entrant.current_splits),
            numstops: lap.numstops,
            penalty: lap.penalty,
        };
        let previous_best = entrant.best_lap;
        let is_personal_best = entrant.best_lap.is_none_or(|b| lap.ltime < b);
        if is_personal_best {
            entrant.best_lap = Some(lap.ltime);
            entrant.best_lap_num = Some(effective_lap);
        }
        entrant.laps_done = effective_lap;
        entrant.laps.push(record.clone());
        let mut events = vec![RaceEvent::LapCompleted {
            id,
            plid: lap.plid,
            record,
        }];
        if is_personal_best {
            events.push(RaceEvent::PersonalBest {
                id,
                plid: lap.plid,
                lap: effective_lap,
                time: lap.ltime,
                previous: previous_best,
            });
        }
        let is_session_fastest = g.fastest_lap.is_none_or(|(_, _, t)| lap.ltime < t);
        if is_session_fastest {
            g.fastest_lap = Some((id, lap.plid, lap.ltime));
            events.push(RaceEvent::FastestLap {
                id,
                plid: lap.plid,
                lap: effective_lap,
                time: lap.ltime,
            });
        }
        events
    }

    /// Record a split crossing. Call on [`insim::Packet::Spx`].
    ///
    /// Ignored once the entrant has finished, and for LFS's invalid-split
    /// placeholder (a split time of 1:00:00.000).
    pub fn apply_split(&self, spx: &Spx) -> Vec<RaceEvent> {
        if spx.stime >= INVALID_SPLIT {
            return vec![];
        }
        let mut g = self.inner.write();
        let Some(&id) = g.live.get(&spx.plid) else {
            return vec![];
        };
        let Some(entrant) = g.entrants.get_mut(&id) else {
            return vec![];
        };
        if matches!(entrant.status, FinishStatus::Finished { .. }) {
            return vec![];
        }
        entrant.current_splits.push(spx.stime);
        vec![RaceEvent::SplitCompleted {
            id,
            plid: spx.plid,
            split: spx.split,
            time: spx.stime,
        }]
    }

    /// Record a provisional finish. Call on [`insim::Packet::Fin`].
    ///
    /// In qualifying, practice and untimed sessions `Fin` fires after *every*
    /// completed lap rather than marking a finish, so it is ignored there. The
    /// finish is only recorded for races (or when no session kind is known, to
    /// stay robust for a tracker started mid-race).
    pub fn apply_finish(&self, fin: &Fin) -> Vec<RaceEvent> {
        let mut g = self.inner.write();
        if matches!(
            g.session_kind,
            Some(SessionKind::Qualifying | SessionKind::Practice | SessionKind::Untimed)
        ) {
            return vec![];
        }
        let Some(&id) = g.live.get(&fin.plid) else {
            return vec![];
        };
        let Some(entrant) = g.entrants.get_mut(&id) else {
            return vec![];
        };
        entrant.status = FinishStatus::Finished {
            ttime: fin.ttime,
            btime: fin.btime,
            numstops: fin.numstops,
            confirm: fin.confirm,
            result_num: None,
        };
        vec![RaceEvent::Finished {
            id,
            plid: fin.plid,
            ttime: fin.ttime,
            btime: fin.btime,
            confirm: fin.confirm,
        }]
    }

    /// Record a confirmed result. Call on [`insim::Packet::Res`].
    ///
    /// `Res` can arrive after the player has already left the track (and so is
    /// no longer in the live map). In that case the classification is applied
    /// to the entrant's preserved record, matched by [`PlayerId`].
    pub fn apply_result(&self, res: &Res) -> Vec<RaceEvent> {
        let mut g = self.inner.write();
        let id = match g.live.get(&res.plid).copied() {
            Some(id) => id,
            None => match find_entrant_by_plid(&g, res.plid) {
                Some(id) => id,
                None => return vec![],
            },
        };
        let Some(entrant) = g.entrants.get_mut(&id) else {
            return vec![];
        };
        if let FinishStatus::Finished { result_num, .. } = &mut entrant.status {
            *result_num = Some(res.resultnum);
        }
        vec![RaceEvent::ResultConfirmed {
            id,
            plid: res.plid,
            result_num: res.resultnum,
            num_results: res.numres,
        }]
    }

    /// Open a pending pit stop record. Call on [`insim::Packet::Pit`].
    pub fn apply_pit_stop(&self, pit: &Pit) -> Vec<RaceEvent> {
        let mut g = self.inner.write();
        let Some(&id) = g.live.get(&pit.plid) else {
            return vec![];
        };
        let Some(entrant) = g.entrants.get_mut(&id) else {
            return vec![];
        };
        entrant.pending_pit = Some(PitRecord {
            stop_number: pit.numstops,
            lap: entrant.laps_done,
            work: pit.work,
            stop_time: None,
        });
        vec![]
    }

    /// Complete the pending pit stop. Call on [`insim::Packet::Psf`].
    pub fn apply_pit_stop_finished(&self, psf: &Psf) -> Vec<RaceEvent> {
        let mut g = self.inner.write();
        let Some(&id) = g.live.get(&psf.plid) else {
            return vec![];
        };
        let Some(entrant) = g.entrants.get_mut(&id) else {
            return vec![];
        };
        let Some(mut record) = entrant.pending_pit.take() else {
            return vec![];
        };
        record.stop_time = Some(psf.stime);
        entrant.pit_stops.push(record.clone());
        vec![RaceEvent::PitStopComplete {
            id,
            plid: psf.plid,
            record,
        }]
    }

    /// Record a penalty change. Call on [`insim::Packet::Pen`].
    pub fn apply_penalty_changed(&self, pen: &Pen) -> Vec<RaceEvent> {
        let mut g = self.inner.write();
        let Some(&id) = g.live.get(&pen.plid) else {
            return vec![];
        };
        let Some(entrant) = g.entrants.get_mut(&id) else {
            return vec![];
        };
        entrant.penalty = pen.newpen;
        vec![RaceEvent::PenaltyChanged {
            id,
            plid: pen.plid,
            oldpen: pen.oldpen,
            newpen: pen.newpen,
            reason: pen.reason.clone(),
        }]
    }
}

/// Returns the [`EntrantId`] of a disconnected entrant (in `entrants` but not
/// `live`) whose last [`DriverRecord`] has the given `uname`.
fn find_disconnected_by_uname(inner: &RaceTrackerInner, uname: &str) -> Option<EntrantId> {
    inner
        .entrants
        .iter()
        .filter(|(id, _)| !inner.live.values().any(|live_id| live_id == *id))
        .find(|(_, e)| e.drivers.last().and_then(|d| d.uname.as_deref()) == Some(uname))
        .map(|(id, _)| *id)
}

/// Returns the [`EntrantId`] of a non-live entrant (in `entrants` but no longer
/// `live`) whose last-known [`PlayerId`] matches `plid`. Used to apply a `Res`
/// that arrives after the player has left the track.
fn find_entrant_by_plid(inner: &RaceTrackerInner, plid: PlayerId) -> Option<EntrantId> {
    inner
        .entrants
        .iter()
        .filter(|(id, _)| !inner.live.values().any(|live_id| live_id == *id))
        .find(|(_, e)| e.plid == plid)
        .map(|(id, _)| *id)
}
