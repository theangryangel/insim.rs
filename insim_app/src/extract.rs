//! Magic-extractor traits and the built-in extractors.
//!
//! Handlers (see [`crate::Handler`]) declare what they care about by listing
//! extractor types as parameters. The dispatcher walks each registered handler
//! per event and invokes it only if every parameter's extractor returns `Some`,
//! so `Packet<Ncn>` / `Event<Connected>` act as type-driven routing keys.

use std::{any::Any, marker::PhantomData};

use tokio::sync::mpsc;

use crate::{
    error::AppError,
    event::{Command, Dispatch},
    extensions::Extensions,
};

/// Context handed to extractors during one dispatch cycle.
#[derive(Debug)]
pub struct ExtractCx<'a, S> {
    /// The event currently being routed.
    pub dispatch: &'a Dispatch,
    /// Shared application state. Extracted by [`State<S>`].
    pub state: &'a S,
    /// Back-channel handle for sending packets / emitting events. Extracted by [`Sender`].
    pub sender: &'a Sender,
    /// Extension registry — populated via [`crate::App::extension`]. Middleware
    /// that wants to be queryable from handlers (e.g. presence) stashes itself
    /// here and looks up its instance from inside its own `FromContext` impl.
    pub extensions: &'a Extensions,
}

/// Trait implemented by every magic-extractor type.
///
/// Returning `None` short-circuits the handler — that's how `Packet<T>` and
/// `Event<T>` act as routing extractors.
pub trait FromContext<S>: Sized + Send {
    /// Try to build `Self` from the current dispatch + state. Return `None` to
    /// skip the handler this cycle (e.g. wrong event type).
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self>;
}

/// Extractor that hands out a clone of the shared application state.
///
/// `S` is expected to be cheap-to-clone (typically a struct of `Arc`s / atomics
/// / channels), matching the axum convention.
#[derive(Debug, Clone)]
pub struct State<S>(pub S);

impl<S: Clone + Send + Sync + 'static> FromContext<S> for State<S> {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        Some(State(cx.state.clone()))
    }
}

/// Trait that lets `Packet<T>` extract a typed wire-packet variant from
/// [`insim::Packet`]. Implementations should match the corresponding variant
/// and clone (cheap — InSim packet structs are small).
///
/// PoC: hand-implemented for the variants the smoke test uses. A macro covering
/// every variant is a follow-up.
pub trait PacketVariant: Sized {
    /// If the given [`insim::Packet`] is this variant, return a reference to it.
    fn extract(p: &insim::Packet) -> Option<&Self>;
}

macro_rules! impl_packet_variant {
    ($($variant:ident),+ $(,)?) => {
        $(
            impl PacketVariant for insim::insim::$variant {
                fn extract(p: &insim::Packet) -> Option<&Self> {
                    match p {
                        insim::Packet::$variant(inner) => Some(inner),
                        _ => None,
                    }
                }
            }
        )+
    };
}

impl_packet_variant!(Ncn, Mso, Cnl, Mtc, Tiny);

/// Routing extractor for wire packets. Returns `Some` only when the current
/// dispatch is a matching [`insim::Packet`] variant.
#[derive(Debug, Clone)]
pub struct Packet<T>(pub T);

impl<T, S> FromContext<S> for Packet<T>
where
    T: PacketVariant + Clone + Send + 'static,
{
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        match cx.dispatch {
            Dispatch::Packet(p) => T::extract(p).cloned().map(Packet),
            _ => None,
        }
    }
}

/// Routing extractor for synthetic events. Returns `Some` only when the current
/// dispatch is a [`Dispatch::Synthetic`] whose inner type matches `T`.
#[derive(Debug, Clone)]
pub struct Event<T>(pub T);

impl<T, S> FromContext<S> for Event<T>
where
    T: Any + Clone + Send + Sync + 'static,
{
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        match cx.dispatch {
            Dispatch::Synthetic(a) => a.downcast_ref::<T>().cloned().map(Event),
            _ => None,
        }
    }
}

/// Back-channel handle for sending packets to LFS or emitting synthetic events.
///
/// Cloneable and cheap; the dispatcher consumes commands from its single shared
/// receiver. Extracted automatically via [`FromContext`].
#[derive(Clone)]
pub struct Sender {
    tx: mpsc::Sender<Command>,
}

impl std::fmt::Debug for Sender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sender").finish_non_exhaustive()
    }
}

impl Sender {
    pub(crate) fn new(tx: mpsc::Sender<Command>) -> Self {
        Self { tx }
    }

    /// Send a packet back to LFS.
    pub async fn send<P: Into<insim::Packet>>(&self, packet: P) -> Result<(), AppError> {
        self.tx
            .send(Command::Send(packet.into()))
            .await
            .map_err(|_| AppError::Closed)
    }

    /// Inject a synthetic event into the dispatch pipeline.
    pub async fn emit<E: Any + Send + Sync + 'static>(&self, event: E) -> Result<(), AppError> {
        self.tx
            .send(Command::Emit(std::sync::Arc::new(event)))
            .await
            .map_err(|_| AppError::Closed)
    }
}

impl<S: Send + Sync> FromContext<S> for Sender {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        Some(cx.sender.clone())
    }
}

// Phantom keeps the lint-friendly footprint when an extractor is unused.
#[allow(dead_code)]
struct _PhantomExtractor<S>(PhantomData<S>);
