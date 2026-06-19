//! Race-tracking logic.
//!
//! This module is private to [`crate::world`]. [`RaceState`] holds the
//! cohesive race data (entrants, the live plid→entrant index, pending grid
//! positions, fastest lap); the packet **handlers** are implemented on
//! [`WorldInner`](crate::world::WorldInner) so they can read sibling state
//! (`game.session_kind`, `connections`) directly via disjoint field borrows,
//! rather than threading it through as parameters. The public API surface is
//! the query methods on [`crate::world::World`].

use std::{collections::HashMap, time::Duration};

use insim::{
    identifiers::{ConnectionId, PlayerId},
    insim::{Fin, Lap, Pen, PenaltyInfo, Pit, Plp, Psf, Reo, Res, Spx},
};

use super::{DriverRecord, EntrantId, EntrantState, FinishStatus, LapRecord, PitRecord, RaceEvent};
use crate::world::{
    WorldInner,
    connection::{ConnectionInfo, PlayerInfo},
};

/// LFS uses 1:00:00.000 as a placeholder for an invalid/missing split time.
const INVALID_SPLIT: Duration = Duration::from_secs(3600);

fn uname_for(
    connections: &HashMap<ConnectionId, ConnectionInfo>,
    ucid: ConnectionId,
) -> Option<String> {
    connections.get(&ucid).map(|c| c.uname.clone())
}

fn pname_for(
    connections: &HashMap<ConnectionId, ConnectionInfo>,
    ucid: ConnectionId,
) -> Option<String> {
    connections.get(&ucid).map(|c| c.pname.clone())
}

#[derive(Default)]
pub(crate) struct RaceState {
    next_id: u64,
    pub(crate) entrants: HashMap<EntrantId, EntrantState>,
    pub(crate) live: HashMap<PlayerId, EntrantId>,
    pending_grid: HashMap<PlayerId, u8>,
    pub(crate) fastest_lap: Option<(EntrantId, PlayerId, Duration)>,
    /// Set by a session-starting `Rst`; the actual clear of the prior session's
    /// data is deferred until the next packet so that the just-ended session's
    /// results stay readable while consumers handle `SessionEnded`. See the
    /// dispatch loop in [`crate::world`].
    pub(crate) pending_reset: bool,
}

impl RaceState {
    fn alloc_id(&mut self) -> EntrantId {
        let id = EntrantId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    /// Returns the [`EntrantId`] of a disconnected entrant (in `entrants` but
    /// not `live`) whose last [`DriverRecord`] has the given `uname`.
    fn find_disconnected_by_uname(&self, uname: &str) -> Option<EntrantId> {
        self.entrants
            .iter()
            .filter(|(id, _)| !self.live.values().any(|live_id| live_id == *id))
            .find(|(_, e)| e.drivers.last().and_then(|d| d.uname.as_deref()) == Some(uname))
            .map(|(id, _)| *id)
    }

    /// Returns the [`EntrantId`] of a non-live entrant whose last-known
    /// [`PlayerId`] matches `plid`. Used to apply a `Res` that arrives after the
    /// player has left the track.
    fn find_entrant_by_plid(&self, plid: PlayerId) -> Option<EntrantId> {
        self.entrants
            .iter()
            .filter(|(id, _)| !self.live.values().any(|live_id| live_id == *id))
            .find(|(_, e)| e.plid == plid)
            .map(|(id, _)| *id)
    }

    /// Clear all per-session race state. Called when a new session starts
    /// (`Rst`), before the new session's entrants are tracked.
    pub(crate) fn clear_for_session(&mut self) {
        self.entrants.clear();
        self.live.clear();
        self.pending_grid.clear();
        self.fastest_lap = None;
    }
}

/// Race packet handlers, driven by the unified dispatch in [`crate::world`].
///
/// These live on [`WorldInner`] (rather than [`RaceState`]) so they can read
/// `self.game.session_kind` and `self.connections` directly while mutating
/// `self.race` - the session/connection context they depend on lives a layer
/// up, and disjoint field borrows let one method touch all three.
impl WorldInner {
    /// The live entrant for `plid`, but only while a tracking session is
    /// active. `None` if not tracking or the plid is not currently on track.
    fn live_entrant_mut(&mut self, plid: PlayerId) -> Option<&mut EntrantState> {
        if !self.game.session_kind.is_some_and(|k| k.is_tracking()) {
            return None;
        }
        let id = *self.race.live.get(&plid)?;
        self.race.entrants.get_mut(&id)
    }

    /// Allocate a fresh entrant for `info`, resolving its display/LFS.net names
    /// from `connections` and applying any buffered grid position. Returns the
    /// new [`EntrantId`]. Shared by the fresh-join and rejoin-no-match paths.
    fn insert_fresh_entrant(&mut self, info: &PlayerInfo) -> EntrantId {
        let id = self.race.alloc_id();
        let uname = uname_for(&self.connections, info.ucid);
        let pname = pname_for(&self.connections, info.ucid).unwrap_or_else(|| info.pname.clone());
        let grid_position = self.race.pending_grid.remove(&info.plid);
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
        let _ = self.race.entrants.insert(id, state);
        let _ = self.race.live.insert(info.plid, id);
        id
    }

    pub(crate) fn on_player_joined(&mut self, info: &PlayerInfo) -> Vec<RaceEvent> {
        if self.rejoin {
            self.apply_player_rejoined(info)
        } else {
            self.apply_player_joined(info)
        }
    }

    fn apply_player_joined(&mut self, info: &PlayerInfo) -> Vec<RaceEvent> {
        if !self.game.session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        // Idempotent per live plid: LFS announces each player with an `Npl` at
        // grid formation *and* again in reply to a `TINY_NPL` request, so the
        // same plid can arrive twice. If it is already live, fold in any pending
        // grid position and ignore the duplicate rather than creating a phantom
        // second entrant.
        if let Some(&id) = self.race.live.get(&info.plid) {
            if let Some(grid) = self.race.pending_grid.remove(&info.plid)
                && let Some(entrant) = self.race.entrants.get_mut(&id)
            {
                entrant.grid_position = Some(grid);
            }
            return vec![];
        }
        let id = self.insert_fresh_entrant(info);
        vec![RaceEvent::EntrantJoined {
            id,
            plid: info.plid,
        }]
    }

    pub(crate) fn on_player_left(&mut self, info: &PlayerInfo) -> Vec<RaceEvent> {
        // Always drop from the live index, even outside a tracking session.
        let Some(id) = self.race.live.remove(&info.plid) else {
            return vec![];
        };
        if self.game.session_kind.is_some_and(|k| k.is_race())
            && let Some(entrant) = self.race.entrants.get_mut(&id)
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

    pub(crate) fn on_taking_over(
        &mut self,
        before: &PlayerInfo,
        after: &PlayerInfo,
    ) -> Vec<RaceEvent> {
        if !self.game.session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        let Some(&id) = self.race.live.get(&before.plid) else {
            return vec![];
        };
        let uname = uname_for(&self.connections, after.ucid);
        let pname = pname_for(&self.connections, after.ucid).unwrap_or_else(|| after.pname.clone());
        if let Some(entrant) = self.race.entrants.get_mut(&id) {
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

    fn apply_player_rejoined(&mut self, info: &PlayerInfo) -> Vec<RaceEvent> {
        if !self.game.session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        // Idempotent per live plid: ignore a duplicate `Npl` for an entrant
        // already on track.
        if let Some(&id) = self.race.live.get(&info.plid) {
            if let Some(grid) = self.race.pending_grid.remove(&info.plid)
                && let Some(entrant) = self.race.entrants.get_mut(&id)
            {
                entrant.grid_position = Some(grid);
            }
            return vec![];
        }
        let uname = uname_for(&self.connections, info.ucid);
        if let Some(prior_id) = uname
            .as_deref()
            .and_then(|u| self.race.find_disconnected_by_uname(u))
        {
            let pname =
                pname_for(&self.connections, info.ucid).unwrap_or_else(|| info.pname.clone());
            let entrant = self.race.entrants.get_mut(&prior_id).unwrap();
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
            let _ = self.race.live.insert(info.plid, prior_id);
            return vec![RaceEvent::EntrantRejoined {
                id: prior_id,
                plid: info.plid,
            }];
        }
        // No match - fresh join
        let id = self.insert_fresh_entrant(info);
        vec![RaceEvent::EntrantJoined {
            id,
            plid: info.plid,
        }]
    }

    pub(crate) fn apply_grid_order(&mut self, reo: &Reo) -> Vec<RaceEvent> {
        if !self.game.session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        let count = (reo.nump as usize).min(reo.plid.len());
        for (i, &plid) in reo.plid.iter().take(count).enumerate() {
            let pos = (i + 1) as u8;
            match self.race.live.get(&plid).copied() {
                Some(id) => {
                    if let Some(entrant) = self.race.entrants.get_mut(&id) {
                        entrant.grid_position = Some(pos);
                    }
                },
                None => {
                    let _ = self.race.pending_grid.insert(plid, pos);
                },
            }
        }
        vec![]
    }

    pub(crate) fn apply_telepit(&mut self, plp: &Plp) -> Vec<RaceEvent> {
        let Some(entrant) = self.live_entrant_mut(plp.plid) else {
            return vec![];
        };
        let id = entrant.id;
        entrant.current_splits.clear();
        entrant.pending_pit = None;
        vec![RaceEvent::TeleportedToPits { id, plid: plp.plid }]
    }

    pub(crate) fn apply_lap(&mut self, lap: &Lap) -> Vec<RaceEvent> {
        let Some(entrant) = self.live_entrant_mut(lap.plid) else {
            return vec![];
        };
        if matches!(entrant.status, FinishStatus::Finished { .. }) {
            return vec![];
        }
        let id = entrant.id;
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
        // `entrant`'s borrow of `*self` ends here; `self.race.fastest_lap` below
        // is a separate field access.
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
        let is_session_fastest = self.race.fastest_lap.is_none_or(|(_, _, t)| lap.ltime < t);
        if is_session_fastest {
            self.race.fastest_lap = Some((id, lap.plid, lap.ltime));
            events.push(RaceEvent::FastestLap {
                id,
                plid: lap.plid,
                lap: effective_lap,
                time: lap.ltime,
            });
        }
        events
    }

    pub(crate) fn apply_split(&mut self, spx: &Spx) -> Vec<RaceEvent> {
        if spx.stime >= INVALID_SPLIT {
            return vec![];
        }
        let Some(entrant) = self.live_entrant_mut(spx.plid) else {
            return vec![];
        };
        if matches!(entrant.status, FinishStatus::Finished { .. }) {
            return vec![];
        }
        let id = entrant.id;
        entrant.current_splits.push(spx.stime);
        vec![RaceEvent::SplitCompleted {
            id,
            plid: spx.plid,
            split: spx.split,
            time: spx.stime,
        }]
    }

    pub(crate) fn apply_finish(&mut self, fin: &Fin) -> Vec<RaceEvent> {
        // Finishes only count in a race; `Fin` fires per-lap in qualifying etc.
        if !self.game.session_kind.is_some_and(|k| k.is_race()) {
            return vec![];
        }
        let Some(&id) = self.race.live.get(&fin.plid) else {
            return vec![];
        };
        let Some(entrant) = self.race.entrants.get_mut(&id) else {
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

    pub(crate) fn apply_result(&mut self, res: &Res) -> Vec<RaceEvent> {
        if !self.game.session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        let id = match self.race.live.get(&res.plid).copied() {
            Some(id) => id,
            None => match self.race.find_entrant_by_plid(res.plid) {
                Some(id) => id,
                None => return vec![],
            },
        };
        let Some(entrant) = self.race.entrants.get_mut(&id) else {
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

    pub(crate) fn apply_pit_stop(&mut self, pit: &Pit) -> Vec<RaceEvent> {
        let Some(entrant) = self.live_entrant_mut(pit.plid) else {
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

    pub(crate) fn apply_pit_stop_finished(&mut self, psf: &Psf) -> Vec<RaceEvent> {
        let Some(entrant) = self.live_entrant_mut(psf.plid) else {
            return vec![];
        };
        let Some(mut record) = entrant.pending_pit.take() else {
            return vec![];
        };
        let id = entrant.id;
        record.stop_time = Some(psf.stime);
        entrant.pit_stops.push(record.clone());
        vec![RaceEvent::PitStopComplete {
            id,
            plid: psf.plid,
            record,
        }]
    }

    pub(crate) fn apply_penalty_changed(&mut self, pen: &Pen) -> Vec<RaceEvent> {
        let Some(entrant) = self.live_entrant_mut(pen.plid) else {
            return vec![];
        };
        let id = entrant.id;
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
