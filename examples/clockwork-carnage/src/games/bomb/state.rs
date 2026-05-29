use std::{
    cmp::Reverse,
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use insim::{
    identifiers::{ConnectionId, PlayerId},
    insim::PlayerType,
};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio_util::sync::CancellationToken;

use super::config::{BombConfig, COLLISION_THRESHOLD_MPS};

#[derive(Clone, Debug)]
pub(super) struct ActiveRun {
    pub(super) started_at: Instant,
    pub(super) deadline: Instant,
    pub(super) current_timeout: Duration,
    pub(super) checkpoints: i64,
    pub(super) uname: String,
    pub(super) pname: String,
    pub(super) ucid: ConnectionId,
}

impl ActiveRun {
    pub(super) fn new(
        uname: String,
        pname: String,
        ucid: ConnectionId,
        config: &BombConfig,
        now: Instant,
    ) -> Self {
        Self {
            started_at: now,
            deadline: now + config.checkpoint_timeout,
            current_timeout: config.checkpoint_timeout,
            checkpoints: 0,
            uname,
            pname,
            ucid,
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
        ucid: ConnectionId,
        uname: String,
    },
    Refreshed {
        ucid: ConnectionId,
        checkpoints: i64,
        new_window: Duration,
    },
    Extended {
        ucid: ConnectionId,
        checkpoints: i64,
        time_left: Duration,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum BombPhase {
    #[default]
    Waiting,
    SettingUp,
    Racing,
}

impl std::fmt::Display for BombPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BombPhase::Waiting => "waiting for players",
            BombPhase::SettingUp => "setting up track",
            BombPhase::Racing => "racing",
        };
        f.write_str(s)
    }
}

#[derive(Clone, Default, Debug)]
pub(super) struct BombGlobal {
    pub(super) phase: BombPhase,
    pub(super) leaderboard: Vec<(String, String, i64, i64)>,
    pub(super) active_runs: Vec<(String, String, i64, Instant, Duration)>,
}

pub(super) struct BombInner {
    pub(super) config: BombConfig,
    pub(super) phase: BombPhase,
    setup_cancel: Option<CancellationToken>,
    pub(super) runtime_cancel: CancellationToken,
    pub(super) active_runs: HashMap<PlayerId, ActiveRun>,
    pub(super) leaderboard: Vec<(String, String, i64, i64)>,
    pub(super) db: Option<(crate::db::Pool, i64)>,
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
                phase: BombPhase::default(),
                setup_cancel: None,
                runtime_cancel: CancellationToken::new(),
                active_runs: HashMap::new(),
                leaderboard: Vec::new(),
                db,
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
    pub(super) fn make_setup_cancel(&mut self) -> CancellationToken {
        let token = self.runtime_cancel.child_token();
        self.setup_cancel = Some(token.clone());
        token
    }

    pub(super) fn cancel_setup(&mut self) {
        if let Some(c) = self.setup_cancel.take() {
            c.cancel();
        }
    }

    pub(super) fn clear_setup_cancel(&mut self) {
        self.setup_cancel = None;
    }

    pub(super) fn on_checkpoint(
        &mut self,
        plid: PlayerId,
        ucid: ConnectionId,
        uname: &str,
        pname: &str,
        ptype: PlayerType,
        is_finish: bool,
        now: Instant,
    ) -> Option<CheckpointOutcome> {
        if let Some(run) = self.active_runs.get_mut(&plid) {
            if run.deadline < now {
                return None;
            }
            let outcome = if is_finish {
                run.current_timeout = self.config.checkpoint_timeout;
                run.deadline = now + self.config.checkpoint_timeout;
                run.checkpoints += 1;
                CheckpointOutcome::Refreshed {
                    ucid: run.ucid,
                    checkpoints: run.checkpoints,
                    new_window: self.config.checkpoint_timeout,
                }
            } else {
                run.deadline = now + run.current_timeout;
                run.current_timeout = run
                    .current_timeout
                    .saturating_sub(self.config.checkpoint_penalty);
                run.checkpoints += 1;
                CheckpointOutcome::Extended {
                    ucid: run.ucid,
                    checkpoints: run.checkpoints,
                    time_left: run.time_left(now),
                }
            };
            return Some(outcome);
        }
        if ptype.contains(PlayerType::AI) {
            return None;
        }
        let _ = self.active_runs.insert(
            plid,
            ActiveRun::new(uname.to_owned(), pname.to_owned(), ucid, &self.config, now),
        );
        Some(CheckpointOutcome::Started {
            ucid,
            uname: uname.to_owned(),
        })
    }

    pub(super) fn on_collision(
        &mut self,
        plid: PlayerId,
        speed_diff_mps: f32,
        now: Instant,
    ) -> Option<(ConnectionId, Duration, Duration)> {
        let fraction = (speed_diff_mps / COLLISION_THRESHOLD_MPS).clamp(0.0, 1.0);
        let penalty = Duration::from_millis(
            (fraction * self.config.collision_max_penalty.as_millis() as f32) as u64,
        );
        if penalty.is_zero() {
            return None;
        }
        let run = self.active_runs.get_mut(&plid)?;
        run.deadline = run.deadline.checked_sub(penalty).unwrap_or(now);
        Some((run.ucid, penalty, run.time_left(now)))
    }

    pub(super) fn on_reset(
        &mut self,
        plid: PlayerId,
        now: Instant,
    ) -> Option<(ConnectionId, Duration, Duration)> {
        let penalty = self.config.checkpoint_penalty;
        let run = self.active_runs.get_mut(&plid)?;
        run.deadline = run.deadline.checked_sub(penalty).unwrap_or(now);
        Some((run.ucid, penalty, run.time_left(now)))
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

    pub(super) fn tick_expired(&mut self, now: Instant) -> Vec<ActiveRun> {
        let expired: Vec<PlayerId> = self
            .active_runs
            .iter()
            .filter(|(_, r)| r.deadline < now)
            .map(|(k, _)| *k)
            .collect();
        expired
            .into_iter()
            .filter_map(|k| self.active_runs.remove(&k))
            .collect()
    }

    pub(super) fn snapshot(&self) -> BombGlobal {
        let mut active: Vec<_> = self
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
        active.sort_by_key(|r| Reverse(r.2));
        BombGlobal {
            phase: self.phase.clone(),
            leaderboard: self.leaderboard.clone(),
            active_runs: active,
        }
    }
}
