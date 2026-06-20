use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use insim::{identifiers::PlayerId, insim::PlayerType};
use kitcar::{PlayerInfo, RoundPhase};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::config::{BombConfig, COLLISION_THRESHOLD_MPS};
use crate::run_registry::RunRegistry;

#[derive(Clone, Debug, Default)]
pub(super) struct CircleSequence(Vec<u8>);

impl CircleSequence {
    pub(super) fn accumulate(&mut self, indices: impl Iterator<Item = u8>) {
        self.0.extend(indices);
    }

    pub(super) fn finalize(&mut self) {
        self.0.sort_unstable();
        self.0.dedup();
    }

    pub(super) fn clear(&mut self) {
        self.0.clear();
    }

    /// Returns true if `candidate` is the circle that should follow `last` in the sequence.
    /// When the sequence is empty (no layout loaded), any candidate is accepted.
    pub(super) fn is_next(&self, last: u8, candidate: u8) -> bool {
        if self.0.is_empty() {
            return true;
        }
        self.0
            .iter()
            .position(|&i| i == last)
            .map(|pos| self.0[(pos + 1) % self.0.len()])
            == Some(candidate)
    }
}

/// In-progress bomb run. Lives in a [`RunRegistry`] keyed by `PlayerId`; the
/// owning `ucid` is resolved from the world when needed (so a driver swap needs
/// no bookkeeping here). `uname`/`pname` are snapshotted because a run can end
/// after the player has already left the world (see [`RunEnded`]).
///
/// [`RunEnded`]: crate::run_registry::RunEnded
#[derive(Clone, Debug)]
pub(super) struct ActiveRun {
    pub(super) started_at: Instant,
    pub(super) deadline: Instant,
    pub(super) current_timeout: Duration,
    pub(super) checkpoints: i64,
    pub(super) uname: String,
    pub(super) pname: String,
    pub(super) last_circle_index: Option<u8>,
}

impl ActiveRun {
    pub(super) fn new(
        uname: String,
        pname: String,
        config: &BombConfig,
        now: Instant,
        first_circle_index: u8,
    ) -> Self {
        Self {
            started_at: now,
            deadline: now + config.checkpoint_timeout,
            current_timeout: config.checkpoint_timeout,
            checkpoints: 0,
            uname,
            pname,
            last_circle_index: Some(first_circle_index),
        }
    }

    pub(super) fn survival_ms(&self, now: Instant) -> i64 {
        (self.deadline.min(now) - self.started_at).as_millis() as i64
    }

    pub(super) fn time_left(&self, now: Instant) -> Duration {
        self.deadline.saturating_duration_since(now)
    }
}

#[derive(Debug)]
pub(super) enum CheckpointOutcome {
    Started {
        uname: String,
    },
    Refreshed {
        checkpoints: i64,
        new_window: Duration,
    },
    Extended {
        checkpoints: i64,
        time_left: Duration,
    },
}

#[derive(Clone, Default, Debug)]
pub(super) struct BombGlobal {
    pub(super) phase: RoundPhase,
    pub(super) leaderboard: Vec<(String, String, i64, i64)>,
    pub(super) active_runs: Vec<(String, String, i64, Instant, Duration)>,
}

pub(super) struct BombInner {
    pub(super) config: BombConfig,
    pub(super) phase: RoundPhase,
    pub(super) leaderboard: Vec<(String, String, i64, i64)>,
    pub(super) db: Option<(crate::db::Pool, i64)>,
    pub(super) circle_sequence: CircleSequence,
}

#[derive(Clone)]
pub(super) struct Bomb {
    inner: Arc<RwLock<BombInner>>,
}

impl Bomb {
    pub(super) fn new(config: BombConfig, db: Option<(crate::db::Pool, i64)>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(BombInner {
                config,
                phase: RoundPhase::default(),
                leaderboard: Vec::new(),
                db,
                circle_sequence: CircleSequence::default(),
            })),
        }
    }

    pub(super) fn read(&self) -> RwLockReadGuard<'_, BombInner> {
        self.inner.read()
    }

    pub(super) fn write(&self) -> RwLockWriteGuard<'_, BombInner> {
        self.inner.write()
    }
}

impl BombInner {
    pub(super) fn accumulate_circles(&mut self, indices: impl Iterator<Item = u8>) {
        self.circle_sequence.accumulate(indices);
    }

    pub(super) fn finalize_circles(&mut self) {
        self.circle_sequence.finalize();
    }

    pub(super) fn clear_circles(&mut self) {
        self.circle_sequence.clear();
    }

    /// Cross an InSim circle. Updates the in-registry run (or starts one); the
    /// run state lives in `runs`, the layout sequence and config live here.
    pub(super) fn on_checkpoint(
        &self,
        runs: &RunRegistry<ActiveRun>,
        uname: &str,
        player: &PlayerInfo,
        circle_index: u8,
        now: Instant,
    ) -> Option<CheckpointOutcome> {
        // Existing run: advance it (or reject an out-of-sequence / expired hit).
        if let Some(outcome) = runs.with_mut(player.plid, |run| {
            if run.deadline < now {
                return None;
            }
            if let Some(last) = run.last_circle_index
                && !self.circle_sequence.is_next(last, circle_index)
            {
                return None;
            }
            run.last_circle_index = Some(circle_index);
            run.deadline = now + run.current_timeout;
            run.current_timeout = run
                .current_timeout
                .saturating_sub(self.config.checkpoint_penalty);
            run.checkpoints += 1;
            Some(CheckpointOutcome::Extended {
                checkpoints: run.checkpoints,
                time_left: run.time_left(now),
            })
        }) {
            return outcome;
        }

        // No run yet: start one (humans only).
        if player.ptype.contains(PlayerType::AI) {
            return None;
        }
        runs.start(
            player.plid,
            ActiveRun::new(
                uname.to_owned(),
                player.pname.clone(),
                &self.config,
                now,
                circle_index,
            ),
        );
        Some(CheckpointOutcome::Started {
            uname: uname.to_owned(),
        })
    }

    pub(super) fn on_time_bonus(
        &self,
        runs: &RunRegistry<ActiveRun>,
        plid: PlayerId,
        now: Instant,
    ) -> Option<CheckpointOutcome> {
        runs.with_mut(plid, |run| {
            if run.deadline < now {
                return None;
            }
            run.current_timeout = self.config.checkpoint_timeout;
            run.deadline = now + self.config.checkpoint_timeout;
            run.checkpoints += 1;
            Some(CheckpointOutcome::Refreshed {
                checkpoints: run.checkpoints,
                new_window: self.config.checkpoint_timeout,
            })
        })
        .flatten()
    }

    pub(super) fn on_collision(
        &self,
        runs: &RunRegistry<ActiveRun>,
        plid: PlayerId,
        speed_diff_mps: f32,
        now: Instant,
    ) -> Option<(Duration, Duration)> {
        let fraction = (speed_diff_mps / COLLISION_THRESHOLD_MPS).clamp(0.0, 1.0);
        let penalty = Duration::from_millis(
            (fraction * self.config.collision_max_penalty.as_millis() as f32) as u64,
        );
        if penalty.is_zero() {
            return None;
        }
        runs.with_mut(plid, |run| {
            run.deadline = run.deadline.checked_sub(penalty).unwrap_or(now);
            (penalty, run.time_left(now))
        })
    }

    pub(super) fn on_reset(
        &self,
        runs: &RunRegistry<ActiveRun>,
        plid: PlayerId,
        now: Instant,
    ) -> Option<(Duration, Duration)> {
        let penalty = self.config.checkpoint_penalty;
        runs.with_mut(plid, |run| {
            run.deadline = run.deadline.checked_sub(penalty).unwrap_or(now);
            (penalty, run.time_left(now))
        })
    }

    pub(super) fn finalize(&mut self, run: &ActiveRun, survival_ms: i64) {
        self.leaderboard.push((
            run.uname.clone(),
            run.pname.clone(),
            run.checkpoints,
            survival_ms,
        ));
        self.leaderboard
            .sort_by(|a, b| b.2.cmp(&a.2).then(b.3.cmp(&a.3)));
        self.leaderboard.truncate(10);
    }
}
