//! Built-in magic-extractor types: [`State`], [`Svc`], [`Packet`], [`Event`].
//!
//! The `FromContext` impls for [`crate::Dispatch`] and
//! [`tokio_util::sync::CancellationToken`] live in the parent module since
//! they don't introduce a new wrapper type.

use std::any::Any;

use super::{ExtractCx, FromContext};
use crate::app::event::Dispatch;

/// Wrapper extractor for any handler value registered via
/// [`crate::App::handle`].
///
/// `Svc<T>` is the cheapest path to extracting a typed handler value
/// without writing a [`FromContext`] impl on `T` yourself: any value
/// whose concrete type is registered (via `app.handle(stage, value)`) is
/// extractable as `Svc<T>` provided it is `Clone + Send + Sync + 'static`.
///
/// Framework-provided stateful handlers ([`crate::World`],
/// [`crate::ui::Ui`]) implement `FromContext` directly so they can be
/// extracted by their own name; user types either do the same or use this
/// wrapper.
#[derive(Debug, Clone)]
pub struct Svc<T>(pub T);

impl<S, V, T> FromContext<S, V> for Svc<T>
where
    V: crate::ui::View + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn from_context(cx: &ExtractCx<'_, S, V>) -> Option<Self> {
        cx.lookup::<T>().map(Svc)
    }
}

/// Extractor for the app's primary state value.
///
/// Available when the app was built via [`crate::App::with_state`]. The
/// framework stores `S` directly and hands out a clone for each extraction.
/// `S` must be `Clone + Send + Sync + 'static`. The framework does **not**
/// wrap your state in any lock; if you need shared mutability, build it
/// into `S` yourself (`Arc<AtomicUsize>` for hot counters, `Arc<Mutex<…>>`
/// or `Arc<RwLock<…>>` for richer mutable bags, plain `Arc<Config>` for
/// read-only data - clone semantics determine what's shared).
#[derive(Debug)]
pub struct State<S>(pub S);

impl<S: Clone> Clone for State<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S> std::ops::Deref for State<S> {
    type Target = S;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> std::ops::DerefMut for State<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S, V> FromContext<S, V> for State<S>
where
    V: crate::ui::View + 'static,
    S: Clone + Send + Sync + 'static,
{
    fn from_context(cx: &ExtractCx<'_, S, V>) -> Option<Self> {
        Some(State(cx.state.clone()))
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

impl<S, V, T> FromContext<S, V> for Packet<T>
where
    V: crate::ui::View + 'static,
    T: Clone + Send + 'static,
    for<'a> &'a T: TryFrom<&'a insim::Packet>,
{
    fn from_context(cx: &ExtractCx<'_, S, V>) -> Option<Self> {
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

impl<S, V, T> FromContext<S, V> for Event<T>
where
    V: crate::ui::View + 'static,
    T: Any + Clone + Send + Sync + 'static,
{
    fn from_context(cx: &ExtractCx<'_, S, V>) -> Option<Self> {
        match cx.dispatch {
            Dispatch::Synthetic(a) => a.downcast_ref::<T>().cloned().map(Event),
            _ => None,
        }
    }
}
