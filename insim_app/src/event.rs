//! Dispatch event types carried through the runtime.

use std::{any::Any, collections::VecDeque, sync::Arc};

/// Internal command sent to the dispatcher through the shared back-channel
/// (via [`crate::Sender`]). The runtime drains this in the main loop.
#[derive(Debug)]
pub(crate) enum Command {
    /// Send a packet to LFS.
    Send(insim::Packet),
    /// Inject a synthetic event into the dispatch queue.
    Emit(Arc<dyn Any + Send + Sync>),
}

/// Synthetic event emitted by the runtime once, immediately after the
/// connection is established and before any wire packets are read.
///
/// Handlers that need to start long-running background tasks (periodic
/// tickers, long polls, etc.) react to this event and `tokio::spawn` from
/// inside the handler body. The spawned task can keep a clone of
/// [`crate::Sender`] to send packets / inject events back into the runtime;
/// when the runtime shuts down, the back-channel closes and any `send` call
/// returns an error, giving the task a natural way to exit.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Startup;

/// An event flowing through the dispatch cycle.
///
/// Wire packets stay typed via [`insim::Packet`]. Synthetic events (emitted by
/// middleware or `Sender::emit` / spawned handlers) ride as an [`Arc<dyn Any>`]
/// so they're cheap to clone and fan out to multiple observers.
#[derive(Clone)]
#[non_exhaustive]
pub enum Dispatch {
    /// A wire packet received from LFS.
    Packet(insim::Packet),
    /// A synthetic event produced inside the runtime.
    Synthetic(Arc<dyn Any + Send + Sync>),
}

impl std::fmt::Debug for Dispatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dispatch::Packet(p) => f.debug_tuple("Packet").field(p).finish(),
            Dispatch::Synthetic(a) => f
                .debug_tuple("Synthetic")
                .field(&(*a).type_id())
                .finish(),
        }
    }
}

/// Handle middleware uses to push synthetic events into the running dispatch cycle.
///
/// Events pushed via `emit` are drained in FIFO order *within the same cycle* — so
/// handlers registered for those synthetic types see them in the same iteration as
/// the originating wire packet (Python-parity semantics).
#[derive(Debug)]
pub struct Emitter<'a> {
    queue: &'a mut VecDeque<Dispatch>,
}

impl<'a> Emitter<'a> {
    pub(crate) fn new(queue: &'a mut VecDeque<Dispatch>) -> Self {
        Self { queue }
    }

    /// Push a synthetic event onto the dispatch queue.
    pub fn emit<E: Any + Send + Sync + 'static>(&mut self, event: E) {
        self.queue.push_back(Dispatch::Synthetic(Arc::new(event)));
    }
}
