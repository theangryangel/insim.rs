//! [`RoundManager`] - the per-round lifecycle engine for round-based
//! mini-games.
//!
//! Owns the `Waiting -> SettingUp -> Racing` state machine, the
//! minimum-players policy, and the [`track_rotation`](crate::track_rotation)
//! orchestration - including the *causation guard* that distinguishes a round
//! genuinely ending from the session churn the manager itself causes while
//! setting up the next round.
//!
//! Register once at [`crate::Stage::Pre`], after [`World`] (so it observes
//! settled session/player state):
//!
//! ```ignore
//! let rounds = RoundManager::new(
//!     RoundPolicy { min_players: 2, setup_timeout: Duration::from_secs(30) },
//!     vec![RoundSpec { track, laps, wind: 0, layout }],
//! );
//! app.handle(Stage::Pre, World::new())
//!    .handle(Stage::Pre, rounds)
//!    .handle(Stage::Update, on_round_started)   // game: announce + reset
//!    .handle(Stage::Update, on_round_ended)     // game: finalize + persist
//!    .handle(Stage::Update, on_gameplay.run_if(|r: RoundManager| r.is_racing()));
//! ```
//!
//! The rotation is a `Vec<RoundSpec>` advanced round-robin, one entry per
//! round: a single-element vec is a static track; many entries rotate. The
//! manager emits [`RoundStarted`] when racing begins and [`RoundEnded`] when a
//! live round ends, then re-arms automatically while enough players remain.

use std::{future::Future, sync::Arc, time::Duration};

use insim::{core::track::Track, insim::RaceLaps};
use insim_extra::world::World;
use parking_lot::RwLock;
use tokio_util::sync::CancellationToken;

use crate::{AppError, ExtractCx, FromContext, Handler, Sender, game::track_rotation};

/// One rotation entry: what to load for a round.
#[derive(Clone, Debug)]
pub struct RoundSpec {
    /// Track to load.
    pub track: Track,
    /// Race length.
    pub laps: RaceLaps,
    /// Wind strength (0..=2 typically).
    pub wind: u8,
    /// Autocross layout to load, if any.
    pub layout: Option<String>,
}

/// Global gating policy, shared across every rotation entry.
#[derive(Clone, Copy, Debug)]
pub struct RoundPolicy {
    /// Minimum connections required before a round may set up.
    pub min_players: usize,
    /// How long to wait for [`track_rotation`](crate::track_rotation) to
    /// confirm the new session before giving up and retrying.
    pub setup_timeout: Duration,
}

/// Lifecycle phase of the current round.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RoundPhase {
    /// Idle - not enough players, or between rounds.
    #[default]
    Waiting,
    /// Driving LFS to load the round's track/layout and (re)start.
    SettingUp,
    /// The round is live.
    Racing,
}

impl std::fmt::Display for RoundPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RoundPhase::Waiting => "waiting for players",
            RoundPhase::SettingUp => "setting up track",
            RoundPhase::Racing => "racing",
        })
    }
}

/// Why a live round ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundEndReason {
    /// The LFS session ended out from under the round (e.g. an admin `/end`).
    SessionEnded,
    /// Connection count dropped below [`RoundPolicy::min_players`].
    NotEnoughPlayers,
}

/// Emitted once when a round's setup completes and racing begins.
#[derive(Clone, Debug)]
pub struct RoundStarted;

/// Emitted once when a live round ends and the manager returns to waiting.
#[derive(Clone, Debug)]
pub struct RoundEnded(pub RoundEndReason);

#[derive(Debug)]
struct RoundInner {
    phase: RoundPhase,
    /// Round-robin position into the rotation.
    cursor: usize,
    /// Bumped on each `SettingUp` entry so a superseded setup task can tell it
    /// is stale and bow out.
    epoch: u64,
    /// Whether a session was active on the previous cycle (for end detection).
    last_session_active: bool,
    setup_cancel: Option<CancellationToken>,
}

/// Per-round lifecycle engine. Clones share one inner state; register the
/// value once and extract it elsewhere (e.g. in a `run_if` gate).
#[derive(Clone, Debug)]
pub struct RoundManager {
    inner: Arc<RwLock<RoundInner>>,
    policy: RoundPolicy,
    rotation: Arc<Vec<RoundSpec>>,
}

impl RoundManager {
    /// Create a manager for `rotation` (advanced round-robin, one entry per
    /// round). An empty rotation never sets up.
    pub fn new(policy: RoundPolicy, rotation: Vec<RoundSpec>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(RoundInner {
                phase: RoundPhase::Waiting,
                cursor: 0,
                epoch: 0,
                last_session_active: false,
                setup_cancel: None,
            })),
            policy,
            rotation: Arc::new(rotation),
        }
    }

    /// The current lifecycle phase.
    pub fn phase(&self) -> RoundPhase {
        self.inner.read().phase
    }

    /// Whether a round is live. Use as a `run_if` gate for gameplay handlers:
    /// `.run_if(|r: RoundManager| r.is_racing())`.
    pub fn is_racing(&self) -> bool {
        self.inner.read().phase == RoundPhase::Racing
    }

    /// Spawn the `track_rotation` for one round. On success promotes to
    /// `Racing` and emits [`RoundStarted`]; on failure/timeout returns to
    /// `Waiting` (the FSM re-arms on a later cycle). Guarded by `epoch` so a
    /// setup that was cancelled and superseded does nothing.
    fn spawn_setup(
        &self,
        world: World,
        sender: Sender,
        spec: RoundSpec,
        cancel: CancellationToken,
        epoch: u64,
    ) {
        let this = self.clone();
        let timeout = self.policy.setup_timeout;
        drop(tokio::spawn(async move {
            let result = tokio::select! {
                r = track_rotation(
                    &world, spec.track, spec.laps, spec.wind, spec.layout.clone(),
                    cancel.clone(), &sender,
                ) => r,
                _ = tokio::time::sleep(timeout) => None,
            };
            let promote = {
                let mut inner = this.inner.write();
                if inner.epoch != epoch || inner.phase != RoundPhase::SettingUp {
                    false // a newer setup superseded this one
                } else {
                    inner.setup_cancel = None;
                    match result {
                        Some(()) => {
                            inner.phase = RoundPhase::Racing;
                            true
                        },
                        None => {
                            inner.phase = RoundPhase::Waiting;
                            false
                        },
                    }
                }
            };
            if promote {
                let _ = sender.event(RoundStarted);
            }
        }));
    }
}

impl<S> FromContext<S> for RoundManager {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        cx.lookup::<RoundManager>()
    }
}

impl<S: Send + Sync + 'static> Handler<(), S> for RoundManager {
    fn call(self, cx: &ExtractCx<'_, S>) -> impl Future<Output = Result<(), AppError>> + Send {
        let world = cx.world.clone();
        let sender = cx.sender.clone();
        let cancel = cx.cancel.clone();
        let mut ended: Option<RoundEndReason> = None;

        {
            let count = world.count();
            let session_active = world.session().is_some();

            // Decide the transition under the lock; perform the spawn after.
            let spawn: Option<(RoundSpec, CancellationToken, u64)> = {
                let mut inner = self.inner.write();
                let spawn = match inner.phase {
                    RoundPhase::Waiting => {
                        if count >= self.policy.min_players && !self.rotation.is_empty() {
                            inner.phase = RoundPhase::SettingUp;
                            inner.epoch = inner.epoch.wrapping_add(1);
                            let epoch = inner.epoch;
                            let spec = self.rotation[inner.cursor % self.rotation.len()].clone();
                            inner.cursor = inner.cursor.wrapping_add(1);
                            let token = cancel.child_token();
                            inner.setup_cancel = Some(token.clone());
                            Some((spec, token, epoch))
                        } else {
                            None
                        }
                    },
                    RoundPhase::SettingUp => {
                        // Ignore the session churn our own setup causes (the
                        // causation guard); only a player shortfall aborts it.
                        if count < self.policy.min_players {
                            if let Some(c) = inner.setup_cancel.take() {
                                c.cancel();
                            }
                            inner.phase = RoundPhase::Waiting;
                        }
                        None
                    },
                    RoundPhase::Racing => {
                        if inner.last_session_active && !session_active {
                            inner.phase = RoundPhase::Waiting;
                            ended = Some(RoundEndReason::SessionEnded);
                        } else if count < self.policy.min_players {
                            inner.phase = RoundPhase::Waiting;
                            ended = Some(RoundEndReason::NotEnoughPlayers);
                        }
                        None
                    },
                };
                inner.last_session_active = session_active;
                spawn
            };

            if let Some((spec, token, epoch)) = spawn {
                self.spawn_setup(world, sender.clone(), spec, token, epoch);
            }
        }

        async move {
            if let Some(reason) = ended {
                let _ = sender.event(RoundEnded(reason));
            }
            Ok(())
        }
    }
}
