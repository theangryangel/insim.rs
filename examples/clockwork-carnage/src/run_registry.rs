//! A per-entrant registry of in-progress "runs" with world-bound lifecycle.
//!
//! Each game defines its own notion of a timed/scored *run* over InSim
//! checkpoint objects (a bomb survival run, a metronome attempt, a shortcut
//! lap). The bookkeeping is identical across them: hold per-player run state
//! keyed by [`PlayerId`], and tear it down when the owning player leaves the
//! track, disconnects, or tele-pits. [`kitcar::World`] already owns that
//! lifecycle, so this registry simply rides its events.
//!
//! Register one per game at [`Stage::Pre`](kitcar::Stage::Pre) and extract it
//! anywhere as `runs: RunRegistry<T>`:
//!
//! - the game starts / queries / finishes runs explicitly ([`start`],
//!   [`with_mut`], [`finish`], [`drain_where`]);
//! - when a player leaves or tele-pits mid-run, the registry removes the run and
//!   emits [`RunEnded<T>`] so the game can score/persist it from the payload
//!   alone (the player may already be gone from the world by then).
//!
//! Lives in clockwork-carnage for now; promote to `kitcar` once all three games
//! are on it.
//!
//! [`start`]: RunRegistry::start
//! [`with_mut`]: RunRegistry::with_mut
//! [`finish`]: RunRegistry::finish
//! [`drain_where`]: RunRegistry::drain_where

use std::{collections::HashMap, future::Future, sync::Arc};

use insim::identifiers::PlayerId;
use kitcar::{
    AppError, Dispatch, ExtractCx, FromContext, Handler, PlayerLeft, PlayerTeleportedToPits,
};
use parking_lot::RwLock;

/// Why a run left the registry on its own (i.e. not via [`RunRegistry::finish`]
/// or [`RunRegistry::drain_where`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunEndReason {
    /// The owning connection left the track or disconnected.
    Left,
    /// The owning player tele-pitted (Shift+P) mid-run.
    TeleportedToPits,
}

/// Emitted when the registry auto-evicts a run. Carries the run state so the
/// game can score/persist it from the payload alone.
#[derive(Clone, Debug)]
pub struct RunEnded<T> {
    /// The player whose run ended.
    pub plid: PlayerId,
    /// The removed run state.
    pub run: T,
    /// Why it ended.
    pub reason: RunEndReason,
}

/// In-progress per-entrant runs of game state `T`, keyed by [`PlayerId`], with
/// lifecycle bound to the world.
pub struct RunRegistry<T> {
    inner: Arc<RwLock<HashMap<PlayerId, T>>>,
    evict_on_pit: bool,
}

impl<T> Clone for RunRegistry<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            evict_on_pit: self.evict_on_pit,
        }
    }
}

impl<T> Default for RunRegistry<T> {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            evict_on_pit: false,
        }
    }
}

impl<T: Clone + Send + Sync + 'static> RunRegistry<T> {
    /// A registry that evicts a run when its owner leaves the track / disconnects.
    pub fn new() -> Self {
        Self::default()
    }

    /// Also evict (and emit [`RunEnded`]) when the owner tele-pits mid-run.
    #[must_use]
    pub fn evict_on_pit(mut self) -> Self {
        self.evict_on_pit = true;
        self
    }

    /// Begin a run for `plid`, replacing any existing one.
    pub fn start(&self, plid: PlayerId, run: T) {
        let _ = self.inner.write().insert(plid, run);
    }

    /// Whether there are no in-progress runs.
    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }

    /// Mutate an in-progress run in place; returns `None` if there isn't one.
    pub fn with_mut<R>(&self, plid: PlayerId, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.inner.write().get_mut(&plid).map(f)
    }

    /// Game-initiated end (finish / timeout): remove and return the run.
    pub fn finish(&self, plid: PlayerId) -> Option<T> {
        self.inner.write().remove(&plid)
    }

    /// Remove and return every run matching `pred` (e.g. expired deadlines).
    pub fn drain_where(&self, pred: impl Fn(&T) -> bool) -> Vec<(PlayerId, T)> {
        let mut inner = self.inner.write();
        let keys: Vec<PlayerId> = inner
            .iter()
            .filter(|(_, v)| pred(v))
            .map(|(k, _)| *k)
            .collect();
        keys.into_iter()
            .filter_map(|k| inner.remove(&k).map(|v| (k, v)))
            .collect()
    }

    /// Snapshot of all in-progress runs (for UI / leaderboard rendering).
    pub fn snapshot(&self) -> Vec<(PlayerId, T)> {
        self.inner
            .read()
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }
}

impl<S, V: kitcar::ui::View + 'static, T: Clone + Send + Sync + 'static> FromContext<S, V>
    for RunRegistry<T>
{
    fn from_context(cx: &ExtractCx<'_, S, V>) -> Option<Self> {
        cx.lookup::<RunRegistry<T>>()
    }
}

/// Rides the world lifecycle: on the (synchronous) `PlayerLeft` /
/// `PlayerTeleportedToPits` dispatch, remove the owner's run and emit
/// [`RunEnded<T>`] for the game to react to.
impl<S: Send + Sync + 'static, V: kitcar::ui::View + 'static, T: Clone + Send + Sync + 'static>
    Handler<(), S, V> for RunRegistry<T>
{
    fn call(self, cx: &ExtractCx<'_, S, V>) -> impl Future<Output = Result<(), AppError>> + Send {
        let evicted = if let Dispatch::Synthetic(payload) = cx.dispatch {
            if let Some(PlayerLeft(info)) = payload.downcast_ref::<PlayerLeft>() {
                self.inner.write().remove(&info.plid).map(|run| RunEnded {
                    plid: info.plid,
                    run,
                    reason: RunEndReason::Left,
                })
            } else if self.evict_on_pit
                && let Some(PlayerTeleportedToPits(info)) =
                    payload.downcast_ref::<PlayerTeleportedToPits>()
            {
                self.inner.write().remove(&info.plid).map(|run| RunEnded {
                    plid: info.plid,
                    run,
                    reason: RunEndReason::TeleportedToPits,
                })
            } else {
                None
            }
        } else {
            None
        };
        let sender = cx.sender.clone();
        async move {
            if let Some(ev) = evicted {
                let _ = sender.event(ev);
            }
            Ok(())
        }
    }
}
