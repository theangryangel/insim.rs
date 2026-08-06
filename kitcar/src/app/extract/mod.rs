//! Magic-extractor framework: the [`ExtractCx`] context and the
//! [`FromContext`] trait every extractor implements.
//!
//! Handlers (see [`crate::Handler`]) declare what they care about by listing
//! extractor types as parameters. The dispatcher walks each registered handler
//! per event and invokes it only if every parameter's extractor returns `Some`,
//! so `Packet<Ncn>` / `Event<Connected>` act as type-driven routing keys.
//!
//! The built-in extractors ([`State`], [`Packet`], [`Event`], plus
//! `FromContext` impls for [`crate::Dispatch`] and
//! [`tokio_util::sync::CancellationToken`]) live in [`builtin`] and are
//! re-exported here.
//!
//! Shared long-lived storage belongs in the bot's primary, type-checked
//! [`State<S>`], set via [`crate::App::with_state`]. Handler-local state can
//! live directly in a manual handler or captured `FnMut` closure.

mod builtin;

pub use builtin::{Event, Packet, State};
use tokio_util::sync::CancellationToken;

use super::{event::Dispatch, runtime::Sender};
use crate::{World, ui::NoView};

/// Context handed to extractors during one dispatch cycle.
///
/// Parameterised by the app's state type `S` and view type `V` (defaulting to
/// `()` and [`NoView`] respectively) - the same two parameters as
/// [`App<S, V>`](crate::App).
pub struct ExtractCx<'a, S = (), V = NoView>
where
    V: crate::ui::View + 'static,
{
    /// The event currently being routed.
    pub dispatch: &'a Dispatch,
    /// Back-channel handle for sending packets / emitting events. Extracted by [`Sender`].
    pub sender: &'a Sender,
    /// The intrinsic world-state mirror for this run. Already folded for the
    /// current dispatch by the runtime's mirror step before any handler runs.
    /// Extracted by `world: World`.
    pub world: &'a World,
    /// The app's UI, intrinsic like `world`. The runtime forwards packets to it
    /// before handlers run; handlers extract it infallibly as `ui: Ui<V>`. For
    /// an app with no UI this is the inert [`NoView`] handle.
    pub(crate) ui: &'a crate::ui::Ui<V>,
    /// Cooperative-shutdown token. Call [`ExtractCx::shutdown`] to request the
    /// runtime exit at its next select iteration.
    pub cancel: &'a CancellationToken,
    /// The app's primary state. Read via the [`State<S>`] extractor (which
    /// clones it for the handler).
    pub state: &'a S,
}

impl<S, V: crate::ui::View + 'static> std::fmt::Debug for ExtractCx<'_, S, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtractCx")
            .field("dispatch", &self.dispatch)
            .finish_non_exhaustive()
    }
}

impl<S, V: crate::ui::View + 'static> ExtractCx<'_, S, V> {
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
///
/// Parameterised by the app's state type `S`. Extractors that don't touch
/// state are implemented for all `S` (i.e. `impl<S> FromContext<S> for Foo`);
/// only [`State<S>`] is bound to a specific state type.
pub trait FromContext<S = (), V = NoView>: Sized + Send
where
    V: crate::ui::View + 'static,
{
    /// Try to build `Self` from the current dispatch context. Return
    /// `None` to skip the handler this cycle (e.g. wrong event type).
    fn from_context(cx: &ExtractCx<'_, S, V>) -> Option<Self>;
}

/// Extractor that hands out a clone of the current [`Dispatch`] regardless
/// of its variant.
impl<S, V: crate::ui::View + 'static> FromContext<S, V> for Dispatch {
    fn from_context(cx: &ExtractCx<'_, S, V>) -> Option<Self> {
        Some(cx.dispatch.clone())
    }
}

/// Extractor that hands out a clone of the runtime's [`CancellationToken`].
impl<S, V: crate::ui::View + 'static> FromContext<S, V> for CancellationToken {
    fn from_context(cx: &ExtractCx<'_, S, V>) -> Option<Self> {
        Some(cx.cancel.clone())
    }
}
