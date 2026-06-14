//! Race-tracking logic as methods on [`RaceState`], with bridge free functions
//! operating on [`super::WorldInner`].
//!
//! This module is private to [`crate::world`]. The public API surface is the
//! query methods on [`super::World`].

use std::{collections::HashMap, time::Duration};

use insim::{
    Packet,
    identifiers::{ConnectionId, PlayerId},
    insim::{Fin, Lap, Pen, PenaltyInfo, Pit, Plp, Psf, Reo, Res, Spx},
};

use super::WorldInner;
use crate::{
    game::SessionKind,
    presence::{ConnectionInfo, PlayerInfo},
    race::{DriverRecord, EntrantId, EntrantState, FinishStatus, LapRecord, PitRecord, RaceEvent},
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
pub(super) struct RaceState {
    next_id: u64,
    pub(super) entrants: HashMap<EntrantId, EntrantState>,
    pub(super) live: HashMap<PlayerId, EntrantId>,
    pending_grid: HashMap<PlayerId, u8>,
    pub(super) fastest_lap: Option<(EntrantId, PlayerId, Duration)>,
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

    pub(super) fn on_player_joined(
        &mut self,
        info: &PlayerInfo,
        connections: &HashMap<ConnectionId, ConnectionInfo>,
        session_kind: Option<SessionKind>,
        rejoin: bool,
    ) -> Vec<RaceEvent> {
        if rejoin {
            self.apply_player_rejoined(info, connections, session_kind)
        } else {
            self.apply_player_joined(info, connections, session_kind)
        }
    }

    fn apply_player_joined(
        &mut self,
        info: &PlayerInfo,
        connections: &HashMap<ConnectionId, ConnectionInfo>,
        session_kind: Option<SessionKind>,
    ) -> Vec<RaceEvent> {
        if !session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        // Idempotent per live plid: LFS announces each player with an `Npl` at
        // grid formation *and* again in reply to a `TINY_NPL` request, so the
        // same plid can arrive twice. If it is already live, fold in any pending
        // grid position and ignore the duplicate rather than creating a phantom
        // second entrant.
        if let Some(&id) = self.live.get(&info.plid) {
            if let Some(grid) = self.pending_grid.remove(&info.plid)
                && let Some(entrant) = self.entrants.get_mut(&id)
            {
                entrant.grid_position = Some(grid);
            }
            return vec![];
        }
        let id = self.alloc_id();
        let uname = uname_for(connections, info.ucid);
        let pname = pname_for(connections, info.ucid).unwrap_or_else(|| info.pname.clone());
        let grid_position = self.pending_grid.remove(&info.plid);
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
        let _ = self.entrants.insert(id, state);
        let _ = self.live.insert(info.plid, id);
        vec![RaceEvent::EntrantJoined {
            id,
            plid: info.plid,
        }]
    }

    pub(super) fn on_player_left(
        &mut self,
        info: &PlayerInfo,
        session_kind: Option<SessionKind>,
    ) -> Vec<RaceEvent> {
        let Some(id) = self.live.remove(&info.plid) else {
            return vec![];
        };
        if session_kind.is_some_and(|k| k.is_race())
            && let Some(entrant) = self.entrants.get_mut(&id)
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

    pub(super) fn on_taking_over(
        &mut self,
        before: &PlayerInfo,
        after: &PlayerInfo,
        connections: &HashMap<ConnectionId, ConnectionInfo>,
        session_kind: Option<SessionKind>,
    ) -> Vec<RaceEvent> {
        if !session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        let Some(&id) = self.live.get(&before.plid) else {
            return vec![];
        };
        let uname = uname_for(connections, after.ucid);
        let pname = pname_for(connections, after.ucid).unwrap_or_else(|| after.pname.clone());
        if let Some(entrant) = self.entrants.get_mut(&id) {
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

    fn apply_player_rejoined(
        &mut self,
        info: &PlayerInfo,
        connections: &HashMap<ConnectionId, ConnectionInfo>,
        session_kind: Option<SessionKind>,
    ) -> Vec<RaceEvent> {
        if !session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        // Idempotent per live plid: ignore a duplicate `Npl` for an entrant
        // already on track.
        if let Some(&id) = self.live.get(&info.plid) {
            if let Some(grid) = self.pending_grid.remove(&info.plid)
                && let Some(entrant) = self.entrants.get_mut(&id)
            {
                entrant.grid_position = Some(grid);
            }
            return vec![];
        }
        let uname = uname_for(connections, info.ucid);
        if let Some(prior_id) = uname
            .as_deref()
            .and_then(|u| self.find_disconnected_by_uname(u))
        {
            let pname = pname_for(connections, info.ucid).unwrap_or_else(|| info.pname.clone());
            let entrant = self.entrants.get_mut(&prior_id).unwrap();
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
            let _ = self.live.insert(info.plid, prior_id);
            return vec![RaceEvent::EntrantRejoined {
                id: prior_id,
                plid: info.plid,
            }];
        }
        // No match - fresh join
        let id = self.alloc_id();
        let pname = pname_for(connections, info.ucid).unwrap_or_else(|| info.pname.clone());
        let grid_position = self.pending_grid.remove(&info.plid);
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
        let _ = self.entrants.insert(id, state);
        let _ = self.live.insert(info.plid, id);
        vec![RaceEvent::EntrantJoined {
            id,
            plid: info.plid,
        }]
    }

    fn apply_grid_order(&mut self, reo: &Reo, session_kind: Option<SessionKind>) -> Vec<RaceEvent> {
        if !session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        let count = (reo.nump as usize).min(reo.plid.len());
        for (i, &plid) in reo.plid.iter().take(count).enumerate() {
            let pos = (i + 1) as u8;
            match self.live.get(&plid).copied() {
                Some(id) => {
                    if let Some(entrant) = self.entrants.get_mut(&id) {
                        entrant.grid_position = Some(pos);
                    }
                },
                None => {
                    let _ = self.pending_grid.insert(plid, pos);
                },
            }
        }
        vec![]
    }

    fn apply_telepit(&mut self, plp: &Plp, session_kind: Option<SessionKind>) -> Vec<RaceEvent> {
        if !session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        let Some(&id) = self.live.get(&plp.plid) else {
            return vec![];
        };
        let Some(entrant) = self.entrants.get_mut(&id) else {
            return vec![];
        };
        entrant.current_splits.clear();
        entrant.pending_pit = None;
        vec![RaceEvent::TeleportedToPits { id, plid: plp.plid }]
    }

    fn apply_lap(&mut self, lap: &Lap, session_kind: Option<SessionKind>) -> Vec<RaceEvent> {
        if !session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        let Some(&id) = self.live.get(&lap.plid) else {
            return vec![];
        };
        let Some(entrant) = self.entrants.get_mut(&id) else {
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
        let is_session_fastest = self.fastest_lap.is_none_or(|(_, _, t)| lap.ltime < t);
        if is_session_fastest {
            self.fastest_lap = Some((id, lap.plid, lap.ltime));
            events.push(RaceEvent::FastestLap {
                id,
                plid: lap.plid,
                lap: effective_lap,
                time: lap.ltime,
            });
        }
        events
    }

    fn apply_split(&mut self, spx: &Spx, session_kind: Option<SessionKind>) -> Vec<RaceEvent> {
        if spx.stime >= INVALID_SPLIT {
            return vec![];
        }
        if !session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        let Some(&id) = self.live.get(&spx.plid) else {
            return vec![];
        };
        let Some(entrant) = self.entrants.get_mut(&id) else {
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

    fn apply_finish(&mut self, fin: &Fin, session_kind: Option<SessionKind>) -> Vec<RaceEvent> {
        if !session_kind.is_some_and(|k| k.is_race()) {
            return vec![];
        }
        let Some(&id) = self.live.get(&fin.plid) else {
            return vec![];
        };
        let Some(entrant) = self.entrants.get_mut(&id) else {
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

    fn apply_result(&mut self, res: &Res, session_kind: Option<SessionKind>) -> Vec<RaceEvent> {
        if !session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        let id = match self.live.get(&res.plid).copied() {
            Some(id) => id,
            None => match self.find_entrant_by_plid(res.plid) {
                Some(id) => id,
                None => return vec![],
            },
        };
        let Some(entrant) = self.entrants.get_mut(&id) else {
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

    fn apply_pit_stop(&mut self, pit: &Pit, session_kind: Option<SessionKind>) -> Vec<RaceEvent> {
        if !session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        let Some(&id) = self.live.get(&pit.plid) else {
            return vec![];
        };
        let Some(entrant) = self.entrants.get_mut(&id) else {
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

    fn apply_pit_stop_finished(
        &mut self,
        psf: &Psf,
        session_kind: Option<SessionKind>,
    ) -> Vec<RaceEvent> {
        if !session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        let Some(&id) = self.live.get(&psf.plid) else {
            return vec![];
        };
        let Some(entrant) = self.entrants.get_mut(&id) else {
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

    fn apply_penalty_changed(
        &mut self,
        pen: &Pen,
        session_kind: Option<SessionKind>,
    ) -> Vec<RaceEvent> {
        if !session_kind.is_some_and(|k| k.is_tracking()) {
            return vec![];
        }
        let Some(&id) = self.live.get(&pen.plid) else {
            return vec![];
        };
        let Some(entrant) = self.entrants.get_mut(&id) else {
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

/// Handle a session starting: clear race state and set session_kind.
pub(super) fn on_session_started(inner: &mut WorldInner, kind: SessionKind) -> Vec<RaceEvent> {
    inner.race.entrants.clear();
    inner.race.live.clear();
    inner.race.pending_grid.clear();
    inner.race.fastest_lap = None;
    inner.session_kind = Some(kind);
    vec![RaceEvent::SessionStarted { kind }]
}

/// Route a raw [`Packet`] to the appropriate race-tracking method.
pub(super) fn apply_packet(inner: &mut WorldInner, packet: &Packet) -> Vec<RaceEvent> {
    let sk = inner.session_kind;
    match packet {
        Packet::Lap(v) => inner.race.apply_lap(v, sk),
        Packet::Spx(v) => inner.race.apply_split(v, sk),
        Packet::Fin(v) => inner.race.apply_finish(v, sk),
        Packet::Res(v) => inner.race.apply_result(v, sk),
        Packet::Pit(v) => inner.race.apply_pit_stop(v, sk),
        Packet::Psf(v) => inner.race.apply_pit_stop_finished(v, sk),
        Packet::Pen(v) => inner.race.apply_penalty_changed(v, sk),
        Packet::Plp(v) => inner.race.apply_telepit(v, sk),
        Packet::Reo(v) => inner.race.apply_grid_order(v, sk),
        _ => vec![],
    }
}
