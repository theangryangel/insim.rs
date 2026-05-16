//! Magic-extractor traits and the built-in extractors.
//!
//! Handlers (see [`crate::Handler`]) declare what they care about by listing
//! extractor types as parameters. The dispatcher walks each registered handler
//! per event and invokes it only if every parameter's extractor returns `Some`,
//! so `Packet<Ncn>` / `Event<Connected>` act as type-driven routing keys.
//!
//! Long-lived state lives in the runtime's *resource registry*. Register a
//! value with [`crate::App::resource`] and extract it from a handler by
//! writing a [`FromContext`] impl on the type itself, or wrap with
//! [`Res<T>`] to extract without writing an impl.

use std::any::Any;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    error::AppError,
    event::{Command, Dispatch},
    extensions::Extensions,
};

/// Context handed to extractors during one dispatch cycle.
#[derive(Debug)]
pub struct ExtractCx<'a> {
    /// The event currently being routed.
    pub dispatch: &'a Dispatch,
    /// Back-channel handle for sending packets / emitting events. Extracted by [`Sender`].
    pub sender: &'a Sender,
    /// Resource registry - populated via [`crate::App::resource`]. Handlers
    /// extract typed values by pulling them out via [`FromContext`].
    pub extensions: &'a Extensions,
    /// Cooperative-shutdown token. Call [`ExtractCx::shutdown`] to request the
    /// runtime exit at its next select iteration.
    pub cancel: &'a CancellationToken,
}

impl<'a> ExtractCx<'a> {
    /// Request graceful shutdown of the runtime.
    pub fn shutdown(&self) {
        self.cancel.cancel();
    }

    /// Whether shutdown has been requested.
    pub fn is_shutdown(&self) -> bool {
        self.cancel.is_cancelled()
    }
}

/// Trait implemented by every magic-extractor type.
///
/// Returning `None` short-circuits the handler - that's how `Packet<T>` and
/// `Event<T>` act as routing extractors.
pub trait FromContext: Sized + Send {
    /// Try to build `Self` from the current dispatch + resources. Return
    /// `None` to skip the handler this cycle (e.g. wrong event type).
    fn from_context(cx: &ExtractCx<'_>) -> Option<Self>;
}

/// Wrapper extractor for any resource registered via [`crate::App::resource`].
///
/// `Res<T>` is the cheapest path to extracting a typed resource from a
/// handler without writing a [`FromContext`] impl on `T` yourself: any value
/// registered with `.resource(value)` is automatically extractable as
/// `Res<T>` provided it is `Clone + Send + Sync + 'static`.
///
/// Framework-provided resources (`Presence`, `Game`, `Ui`, …) implement
/// `FromContext` directly so they can be extracted by their own name; user
/// types either do the same or use this wrapper.
#[derive(Debug, Clone)]
pub struct Res<T>(pub T);

impl<T> FromContext for Res<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn from_context(cx: &ExtractCx<'_>) -> Option<Self> {
        cx.extensions.get::<T>().map(Res)
    }
}

/// Routing extractor for wire packets. Returns `Some` only when the current
/// dispatch is a matching [`insim::Packet`] variant.
///
/// The variant-extraction machinery is provided by stdlib's
/// [`TryFrom`]: every `insim::Packet` variant comes with a
/// `TryFrom<&insim::Packet> for &Variant` impl emitted by `insim`'s own
/// `define_packet!` macro, so this extractor automatically covers every
/// variant the protocol defines - no hand-maintained list.
#[derive(Debug, Clone)]
pub struct Packet<T>(pub T);

impl<T> FromContext for Packet<T>
where
    T: Clone + Send + 'static,
    for<'a> &'a T: TryFrom<&'a insim::Packet>,
{
    fn from_context(cx: &ExtractCx<'_>) -> Option<Self> {
        match cx.dispatch {
            Dispatch::Packet(p) => <&T>::try_from(p).ok().cloned().map(Packet),
            _ => None,
        }
    }
}

/// Routing extractor for synthetic events. Returns `Some` only when the current
/// dispatch is a [`Dispatch::Synthetic`] whose inner type matches `T`.
#[derive(Debug, Clone)]
pub struct Event<T>(pub T);

impl<T> FromContext for Event<T>
where
    T: Any + Clone + Send + Sync + 'static,
{
    fn from_context(cx: &ExtractCx<'_>) -> Option<Self> {
        match cx.dispatch {
            Dispatch::Synthetic(a) => a.downcast_ref::<T>().cloned().map(Event),
            _ => None,
        }
    }
}

/// Back-channel handle to the runtime.
///
/// Cloneable, cheap, and unbounded - sends never block (we trade backpressure
/// for freedom from a deadlock window where the dispatch task is itself the
/// only thing that can drain the channel). Two operations:
///
/// - [`Sender::packet`] - push a wire packet out to LFS.
/// - [`Sender::event`]  - inject a synthetic event into a new dispatch cycle.
///
/// Both routes end up at the same receiver in the dispatcher's main loop.
/// **Emission semantics: regardless of caller (resource handler, spawned
/// task, anywhere), events posted here are processed in a *subsequent*
/// dispatch cycle - not the current one.** This is the only emission API
/// in the crate.
#[derive(Clone)]
pub struct Sender {
    tx: mpsc::UnboundedSender<Command>,
}

impl std::fmt::Debug for Sender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sender").finish_non_exhaustive()
    }
}

impl Sender {
    pub(crate) fn new(tx: mpsc::UnboundedSender<Command>) -> Self {
        Self { tx }
    }

    /// Send a packet back to LFS. Non-blocking; only errors if the runtime
    /// has shut down (back-channel closed).
    pub fn packet<P: Into<insim::Packet>>(&self, packet: P) -> Result<(), AppError> {
        self.tx
            .send(Command::Packet(packet.into()))
            .map_err(|_| AppError::Closed)
    }

    /// Inject a synthetic event into a new dispatch cycle. Non-blocking; only
    /// errors if the runtime has shut down.
    pub fn event<E: Any + Send + Sync + 'static>(&self, event: E) -> Result<(), AppError> {
        self.tx
            .send(Command::Event(std::sync::Arc::new(event)))
            .map_err(|_| AppError::Closed)
    }
}

impl FromContext for Sender {
    fn from_context(cx: &ExtractCx<'_>) -> Option<Self> {
        Some(cx.sender.clone())
    }
}

/// Extractor that hands out a clone of the current [`Dispatch`] regardless
/// of its variant. Use for handlers that observe **every** dispatch and want
/// to pattern-match internally - the typed `Packet<T>` / `Event<T>`
/// extractors are preferable when you only care about one variant.
impl FromContext for Dispatch {
    fn from_context(cx: &ExtractCx<'_>) -> Option<Self> {
        Some(cx.dispatch.clone())
    }
}

/// Extractor that hands out a clone of the runtime's [`CancellationToken`].
///
/// Lets handlers trigger graceful shutdown (`token.cancel()`) or check whether
/// shutdown is already in progress (`token.is_cancelled()`) without needing a
/// custom `Handler` impl just to read [`ExtractCx::cancel`].
impl FromContext for CancellationToken {
    fn from_context(cx: &ExtractCx<'_>) -> Option<Self> {
        Some(cx.cancel.clone())
    }
}
