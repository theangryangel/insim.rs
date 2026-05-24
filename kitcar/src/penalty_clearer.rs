//! [`PenaltyClearer`] - queues deferred penalty clears and drains them once
//! a configurable delay has elapsed.
//!
//! Register at [`crate::Stage::Pre`] alongside [`crate::Presence`]:
//!
//! ```ignore
//! app.handle(Stage::Pre, PenaltyClearer::new(Duration::from_secs(15)))
//! ```
//!
//! On each dispatch cycle the `Handler` impl:
//! - Queues a deferred clear when a `Pen` packet arrives with a non-`Unknown`
//!   reason, resolved to a `ConnectionId` via [`Presence`].
//! - Drains any entries whose delay has elapsed and issues `/p_clear` for each.
//!
//! Call [`PenaltyClearer::clear`] on round reset to discard stale entries.

use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use insim::{identifiers::ConnectionId, insim::PenaltyReason};

use crate::{AppError, Dispatch, ExtractCx, FromContext, Handler, Presence};

/// Service that queues deferred penalty clears and drains them after a fixed
/// delay. On each dispatch cycle it queues incoming `Pen` packets and clears
/// expired entries via [`Presence`].
///
/// Clones are cheap - all share the same inner map.
#[derive(Clone, Debug)]
pub struct PenaltyClearer {
    inner: Arc<RwLock<HashMap<ConnectionId, Instant>>>,
    delay: Duration,
}

impl PenaltyClearer {
    /// Create a new clearer. Queued entries will be drained and cleared after
    /// `delay` has elapsed.
    pub fn new(delay: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            delay,
        }
    }

    /// Queue a deferred clear for `ucid`. If one is already pending the
    /// existing timestamp is kept - the clear fires relative to the first
    /// queued penalty, not the most recent.
    pub fn queue(&self, ucid: ConnectionId) {
        let _ = self
            .inner
            .write()
            .expect("poison")
            .entry(ucid)
            .or_insert_with(Instant::now);
    }

    /// Discard all pending clears without issuing them. Call this on round
    /// reset so stale entries don't leak into the next round.
    pub fn clear(&self) {
        self.inner.write().expect("poison").clear();
    }

    fn drain_expired(&self, now: Instant) -> Vec<ConnectionId> {
        let mut guard = self.inner.write().expect("poison");
        let expired: Vec<ConnectionId> = guard
            .iter()
            .filter(|(_, t)| now.duration_since(**t) >= self.delay)
            .map(|(ucid, _)| *ucid)
            .collect();
        for ucid in &expired {
            let _ = guard.remove(ucid);
        }
        expired
    }
}

impl<S> FromContext<S> for PenaltyClearer {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        cx.lookup::<PenaltyClearer>()
    }
}

impl<S: Send + Sync + 'static> Handler<(), S> for PenaltyClearer {
    fn call(self, cx: &ExtractCx<'_, S>) -> impl Future<Output = Result<(), AppError>> + Send {
        let presence = cx.lookup::<Presence>();
        let sender = cx.sender.clone();

        // Queue incoming penalties before draining so a penalty that arrives
        // exactly at the delay boundary doesn't get cleared in the same cycle.
        if let Dispatch::Packet(insim::Packet::Pen(pen)) = cx.dispatch
            && !matches!(pen.reason, PenaltyReason::Unknown)
            && let Some(ref presence) = presence
            && let Some(conn) = presence.connection_by_player(pen.plid)
        {
            self.queue(conn.ucid);
        }

        async move {
            let Some(presence) = presence else {
                return Ok(());
            };
            for ucid in self.drain_expired(Instant::now()) {
                if let Some(packet) = presence.clear_penalty(ucid) {
                    let _ = sender.packet(packet);
                }
            }
            Ok(())
        }
    }
}
