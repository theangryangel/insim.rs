//! Dispatch event types carried through the runtime.

use std::{any::Any, sync::Arc};

/// An event flowing through the dispatch cycle.
///
/// Wire packets stay typed via [`insim::Packet`]. Synthetic events (emitted
/// via [`crate::Sender::event`] from anywhere - extensions, handlers, the UI
/// thread) ride as an [`Arc<dyn Any>`] so they're cheap to clone.
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
            Dispatch::Synthetic(a) => f.debug_tuple("Synthetic").field(&(*a).type_id()).finish(),
        }
    }
}

/// Internal command sent to the dispatcher through the shared back-channel
/// (via [`crate::Sender`]). The runtime drains this in its main loop.
#[derive(Debug)]
pub(crate) enum Command {
    /// Send a packet to LFS.
    Packet(insim::Packet),
    /// Inject a synthetic event into a new dispatch cycle.
    Event(Arc<dyn Any + Send + Sync>),
}

/// Synthetic event emitted by the runtime once, immediately after the
/// connection is established and before any wire packets are read.
///
/// Handlers that need to start long-running background tasks (periodic
/// tickers, long polls, etc.) react to this event and `tokio::spawn` from
/// inside the handler body.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Startup;
