//! Magic-extractor framework: the [`ExtractCx`] context and the
//! [`FromContext`] trait every extractor implements.
//!
//! Handlers (see [`crate::Handler`]) declare what they care about by listing
//! extractor types as parameters. The dispatcher walks each registered handler
//! per event and invokes it only if every parameter's extractor returns `Some`,
//! so `Packet<Ncn>` / `Event<Connected>` act as type-driven routing keys.
//!
//! The built-in extractors ([`State`], [`Svc`], [`Packet`], [`Event`], plus
//! `FromContext` impls for [`crate::Dispatch`] and
//! [`tokio_util::sync::CancellationToken`]) live in [`builtin`] and are
//! re-exported here.
//!
//! Long-lived storage lives in one of two places:
//!
//! - **[`State<S>`]** - the bot's primary, type-checked state value. Set
//!   once via [`crate::App::with_state`]; extract via [`State`].
//! - **The handler maps** - every handler registered via
//!   [`crate::App::handle`] is also inserted into a TypeId-keyed map per
//!   stage. Handlers (and any value that impls `Handler`, including
//!   passive data types via the trait's default no-op `call`) are
//!   extractable by their concrete type via [`FromContext`] / [`Svc<T>`].

mod builtin;

pub use builtin::{Event, Packet, State, Svc};
use indexmap::IndexMap;
use tokio_util::sync::CancellationToken;

use super::{event::Dispatch, handler::ErasedHandler, runtime::Sender};
use crate::World;

/// Context handed to extractors during one dispatch cycle.
///
/// Parameterised by the app's state type `S`. For stateless apps the default
/// `S = ()` is used; for stateful apps built with [`crate::App::with_state`],
/// `S` is the value passed in.
pub struct ExtractCx<'a, S = ()> {
    /// The event currently being routed.
    pub dispatch: &'a Dispatch,
    /// Back-channel handle for sending packets / emitting events. Extracted by [`Sender`].
    pub sender: &'a Sender,
    /// The intrinsic world-state mirror for this run. Already folded for the
    /// current dispatch by the runtime's mirror step before any handler runs.
    /// Extracted by `world: World`.
    pub world: &'a World,
    /// The app's optional UI, if one was registered via
    /// [`App::with_ui`](crate::App::with_ui). The runtime forwards packets to it
    /// before handlers run; handlers extract the concrete `Ui<V>` from it.
    pub(crate) ui: Option<&'a dyn crate::ui::UiSink>,
    /// Pre-stage handlers, also serving as the typed registry for extraction
    /// (looked up by `TypeId`).
    pub(crate) pre_handlers: &'a IndexMap<std::any::TypeId, Box<dyn ErasedHandler<S>>>,
    /// Update-stage handlers, ditto.
    pub(crate) update_handlers: &'a IndexMap<std::any::TypeId, Box<dyn ErasedHandler<S>>>,
    /// Cooperative-shutdown token. Call [`ExtractCx::shutdown`] to request the
    /// runtime exit at its next select iteration.
    pub cancel: &'a CancellationToken,
    /// The app's primary state. Read via the [`State<S>`] extractor (which
    /// clones it for the handler).
    pub state: &'a S,
}

impl<S> std::fmt::Debug for ExtractCx<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtractCx")
            .field("dispatch", &self.dispatch)
            .field("pre_handlers", &self.pre_handlers.len())
            .field("update_handlers", &self.update_handlers.len())
            .finish_non_exhaustive()
    }
}

impl<S> ExtractCx<'_, S> {
    /// Request graceful shutdown of the runtime.
    pub fn shutdown(&self) {
        self.cancel.cancel();
    }

    /// Whether shutdown has been requested.
    pub fn is_shutdown(&self) -> bool {
        self.cancel.is_cancelled()
    }

    /// Look up a registered handler value by its concrete type and return a
    /// clone. Searches both the pre-stage and update-stage maps. Returns
    /// `None` if no handler of type `T` has been registered.
    ///
    /// This is the mechanism `FromContext` impls on stateful handler types
    /// use; users normally extract via that path (`presence: Presence` in a
    /// handler signature) rather than calling `lookup` directly.
    pub fn lookup<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        let tid = std::any::TypeId::of::<T>();
        let entry = self
            .pre_handlers
            .get(&tid)
            .or_else(|| self.update_handlers.get(&tid))?;
        entry.handler_as_any().downcast_ref::<T>().cloned()
    }
}

/// Trait implemented by every magic-extractor type.
///
/// Returning `None` short-circuits the handler - that's how `Packet<T>` and
/// `Event<T>` act as routing extractors.
///
/// Parameterised by the app's state type `S`. Extractors that don't touch
/// state are implemented for all `S` (i.e. `impl<S> FromContext<S> for Foo`);
/// only [`State<S>`] is bound to a specific state type.
pub trait FromContext<S = ()>: Sized + Send {
    /// Try to build `Self` from the current dispatch context. Return
    /// `None` to skip the handler this cycle (e.g. wrong event type).
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self>;
}

/// Extractor that hands out a clone of the current [`Dispatch`] regardless
/// of its variant.
impl<S> FromContext<S> for Dispatch {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        Some(cx.dispatch.clone())
    }
}

/// Extractor that hands out a clone of the runtime's [`CancellationToken`].
impl<S> FromContext<S> for CancellationToken {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        Some(cx.cancel.clone())
    }
}
