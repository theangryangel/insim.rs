use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use insim::{
    core::vehicle::Vehicle,
    identifiers::{ConnectionId, PlayerId},
};

use crate::hud::BombLeaderboard;

/// Configuration for the bomb game.
#[derive(Debug, Clone, Copy)]
pub struct BombConfig {
    pub checkpoint_timeout: Duration,
    pub checkpoint_penalty: Duration,
    pub collision_max_penalty: Duration,
}

/// A player's active run in the bomb game.
#[derive(Debug, Clone)]
pub struct ActiveRun {
    pub started_at: Instant,
    pub deadline: Instant,
    pub current_timeout: Duration,
    pub checkpoints: i64,
    pub vehicle: Vehicle,
    pub uname: String,
    pub pname: String,
    pub ucid: ConnectionId,
}

impl ActiveRun {
    pub fn new(
        uname: String,
        pname: String,
        vehicle: Vehicle,
        ucid: ConnectionId,
        config: &BombConfig,
        now: Instant,
    ) -> Self {
        Self {
            started_at: now,
            deadline: now + config.checkpoint_timeout,
            current_timeout: config.checkpoint_timeout,
            checkpoints: 0,
            vehicle,
            uname,
            pname,
            ucid,
        }
    }

    /// Calculates survival time in milliseconds, capped at now if the run is ongoing.
    pub fn survival_ms(&self, now: Instant) -> i64 {
        (self.deadline.min(now) - self.started_at).as_millis() as i64
    }

    pub fn time_left(&self, now: Instant) -> Duration {
        self.deadline.saturating_duration_since(now)
    }
}

/// The result of a run completion (death, leave, pit, etc.).
#[derive(Debug)]
#[allow(dead_code)]
pub struct RunResult {
    pub run: ActiveRun,
    pub reason: RunEndReason,
}

#[derive(Debug, Clone, Copy)]
pub enum RunEndReason {
    Exploded, // Timer ran out
    LeftRace, // Player left server or spectated
    Pitted,   // Player entered pits
}

/// The result of hitting a checkpoint.
#[derive(Debug)]
pub enum CheckpointResult {
    Refreshed {
        ucid: ConnectionId,
        checkpoints: i64,
        new_window: Duration,
    },
    Extended {
        ucid: ConnectionId,
        checkpoints: i64,
        penalty: Duration,
        time_left: Duration,
    },
    Started {
        ucid: ConnectionId,
    },
}

/// The result of a collision penalty.
#[derive(Debug)]
pub struct PenaltyResult {
    pub ucid: ConnectionId,
    pub penalty: Duration,
    pub time_left: Duration,
}

/// Manages the state of the Bomb game (active runs, leaderboard, settings).
pub struct BombState {
    pub config: BombConfig,
    pub leaderboard: BombLeaderboard,
    /// Active runs keyed by PlayerId — the canonical in-race identity.
    pub active_runs: HashMap<PlayerId, ActiveRun>,
}

impl BombState {
    pub fn new(config: BombConfig, leaderboard: BombLeaderboard) -> Self {
        Self {
            config,
            leaderboard,
            active_runs: HashMap::new(),
        }
    }

    /// Handles a player hitting a checkpoint.
    /// Returns `Some(CheckpointResult)` if it affected a run or started a new one.
    #[allow(clippy::too_many_arguments)]
    pub fn on_checkpoint(
        &mut self,
        uname: String,
        pname: String,
        ucid: ConnectionId,
        plid: PlayerId,
        vehicle: Vehicle,
        is_finish: bool,
        now: Instant,
    ) -> Option<CheckpointResult> {
        if let Some(run) = self.active_runs.get_mut(&plid) {
            let result = if is_finish {
                run.current_timeout = self.config.checkpoint_timeout;
                run.deadline = now + self.config.checkpoint_timeout;
                run.checkpoints += 1;
                CheckpointResult::Refreshed {
                    ucid,
                    checkpoints: run.checkpoints,
                    new_window: self.config.checkpoint_timeout,
                }
            } else {
                run.deadline = now + run.current_timeout;
                run.current_timeout = run
                    .current_timeout
                    .saturating_sub(self.config.checkpoint_penalty);
                run.checkpoints += 1;
                CheckpointResult::Extended {
                    ucid,
                    checkpoints: run.checkpoints,
                    penalty: self.config.checkpoint_penalty,
                    time_left: run.time_left(now),
                }
            };
            return Some(result);
        }

        let _ = self.active_runs.insert(
            plid,
            ActiveRun::new(uname, pname, vehicle, ucid, &self.config, now),
        );
        Some(CheckpointResult::Started { ucid })
    }

    /// Handles a player leaving the race by PlayerId (Pll/Plp packets).
    pub fn on_leave(&mut self, plid: PlayerId) -> Option<RunResult> {
        self.active_runs.remove(&plid).map(|run| RunResult {
            run,
            reason: RunEndReason::LeftRace,
        })
    }

    /// Handles a driver swap (Toc packet) — updates the stored ucid for the run.
    pub fn on_toc(&mut self, plid: PlayerId, new_ucid: ConnectionId) {
        if let Some(run) = self.active_runs.get_mut(&plid) {
            run.ucid = new_ucid;
        }
    }

    pub fn on_pit(&mut self, plid: PlayerId) -> Option<RunResult> {
        self.active_runs.remove(&plid).map(|run| RunResult {
            run,
            reason: RunEndReason::Pitted,
        })
    }

    /// Handles a collision penalty.
    pub fn on_collision(
        &mut self,
        plid: PlayerId,
        speed_diff: f32, // meters per second
        now: Instant,
    ) -> Option<PenaltyResult> {
        const THRESHOLD_MPS: f32 = 30.0;
        let fraction = (speed_diff / THRESHOLD_MPS).clamp(0.0, 1.0);
        let penalty = Duration::from_millis(
            (fraction * self.config.collision_max_penalty.as_millis() as f32) as u64,
        );

        if penalty.is_zero() {
            return None;
        }

        if let Some(run) = self.active_runs.get_mut(&plid) {
            run.deadline -= penalty;
            return Some(PenaltyResult {
                ucid: run.ucid,
                penalty,
                time_left: run.time_left(now),
            });
        }
        None
    }

    /// Handle a car reset penalty.
    pub fn on_reset(&mut self, plid: PlayerId, now: Instant) -> Option<PenaltyResult> {
        if let Some(run) = self.active_runs.get_mut(&plid) {
            let penalty = self.config.checkpoint_penalty;
            run.deadline -= penalty;
            return Some(PenaltyResult {
                ucid: run.ucid,
                penalty,
                time_left: run.time_left(now),
            });
        }
        None
    }

    /// Returns active runs sorted by checkpoints desc, then deadline desc — for HUD display.
    pub fn active_runs_props(&self) -> Vec<(String, String, i64, Instant, Duration)> {
        let mut runs: Vec<_> = self
            .active_runs
            .values()
            .map(|r| {
                (
                    r.uname.clone(),
                    r.pname.clone(),
                    r.checkpoints,
                    r.deadline,
                    r.current_timeout,
                )
            })
            .collect();
        runs.sort_by(|a, b| b.2.cmp(&a.2).then(b.3.cmp(&a.3)));
        runs
    }

    /// Checks for expired runs.
    pub fn tick(&mut self, now: Instant) -> Vec<RunResult> {
        let keys: Vec<PlayerId> = self
            .active_runs
            .iter()
            .filter(|(_, run)| run.deadline < now)
            .map(|(k, _)| *k)
            .collect();

        keys.into_iter()
            .filter_map(|k| {
                self.active_runs.remove(&k).map(|run| RunResult {
                    run,
                    reason: RunEndReason::Exploded,
                })
            })
            .collect()
    }
}
