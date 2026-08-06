//! [`PenaltyClearer`] - queues deferred penalty clears and drains them once
//! a configurable delay has elapsed.
//!
//! Register like any other handler:
//!
//! ```ignore
//! app.handle(PenaltyClearer::new(Duration::from_secs(15)))
//! ```
//!
//! The `Handler` impl:
//! - Queues a deferred clear when a `Pen` packet arrives with a non-`Unknown`
//!   reason, resolved to a `ConnectionId` via [`World`].
//! - Schedules a private synthetic tick for the deadline, so clearing does not
//!   depend on unrelated traffic arriving later.
//! - Discards pending clears when a session ends or a new one starts.

use std::{
    collections::HashMap,
    future::Future,
    time::{Duration, Instant},
};

use insim::{identifiers::ConnectionId, insim::PenaltyReason};

use crate::{AppError, Dispatch, ExtractCx, Handler, SessionEnded, SessionStarted};

#[derive(Clone, Copy, Debug)]
struct PenaltyClearTick;

/// Mutable handler that queues deferred penalty clears and drains them after a
/// fixed delay.
#[derive(Debug)]
pub struct PenaltyClearer {
    pending: HashMap<ConnectionId, Instant>,
    delay: Duration,
}

impl PenaltyClearer {
    /// Create a new clearer. Queued entries will be drained and cleared after
    /// `delay` has elapsed.
    pub fn new(delay: Duration) -> Self {
        Self {
            pending: HashMap::new(),
            delay,
        }
    }

    fn drain_expired(&mut self, now: Instant) -> Vec<ConnectionId> {
        let expired: Vec<ConnectionId> = self
            .pending
            .iter()
            .filter(|(_, t)| now.duration_since(**t) >= self.delay)
            .map(|(ucid, _)| *ucid)
            .collect();
        for ucid in &expired {
            let _ = self.pending.remove(ucid);
        }
        expired
    }
}

impl<S: Send + Sync + 'static, V: crate::ui::View + 'static> Handler<(), S, V> for PenaltyClearer {
    fn call(
        &mut self,
        cx: &ExtractCx<'_, S, V>,
    ) -> impl Future<Output = Result<(), AppError>> + Send {
        let world = cx.world.clone();
        let sender = cx.sender.clone();
        let cancel = cx.cancel.clone();

        if let Dispatch::Synthetic(event) = cx.dispatch
            && (event.is::<SessionEnded>() || event.is::<SessionStarted>())
        {
            self.pending.clear();
        }

        if let Dispatch::Packet(insim::Packet::Pen(pen)) = cx.dispatch
            && !matches!(pen.reason, PenaltyReason::Unknown)
            && let Some(conn) = world.connection_by_player(pen.plid)
        {
            use std::collections::hash_map::Entry;
            if let Entry::Vacant(entry) = self.pending.entry(conn.ucid) {
                let _ = entry.insert(Instant::now());
                let sender = sender.clone();
                let delay = self.delay;
                drop(tokio::spawn(async move {
                    tokio::select! {
                        _ = cancel.cancelled() => {},
                        _ = tokio::time::sleep(delay) => {
                            let _ = sender.event(PenaltyClearTick);
                        },
                    }
                }));
            }
        }

        let should_drain = matches!(
            cx.dispatch,
            Dispatch::Synthetic(event) if event.is::<PenaltyClearTick>()
        );
        let expired = if should_drain {
            self.drain_expired(Instant::now())
        } else {
            Vec::new()
        };

        async move {
            for ucid in expired {
                if let Some(packet) = world.connection(ucid).map(|c| c.clear_penalty()) {
                    let _ = sender.packet(packet);
                }
            }
            Ok(())
        }
    }
}
