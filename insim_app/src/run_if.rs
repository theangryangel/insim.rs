//! Conditional handler gating via `run_if(predicate)`.
//!
//! Today's implicit gating only routes on *extractor type* — `Packet<Uco>`
//! runs when the dispatch is a `Uco`, `Event<MyTick>` runs when one is
//! injected. That's a *what* gate. `run_if` adds a *when* gate: predicate the
//! same handler on the current state of the world.
//!
//! ```ignore
//! use insim_app::{HandlerExt, in_state};
//!
//! app.handler(on_uco.run_if(in_state(|p: &Phase| p.is_running())))
//! ```
//!
//! Predicates are `Fn(&ExtractCx<'_>) -> bool` closures — same context
//! handlers see, sync, no `await`. Compose with [`not`] / [`and`] / [`or`].
//!
//! The wrapper short-circuits **before** the inner handler's extractors run,
//! so a gated `Packet<Uco>` handler costs nothing extra when the predicate is
//! `false`.

use std::marker::PhantomData;

use crate::{
    error::AppError,
    extract::{ExtractCx, FromContext},
    handler::Handler,
};

/// A condition evaluated against the current dispatch context to decide
/// whether a wrapped handler should run this cycle.
///
/// Any `Fn(&ExtractCx<'_>) -> bool` closure that is `Clone + Send + Sync +
/// 'static` automatically implements this trait, so callers write predicates
/// as plain closures and rarely name the trait directly.
pub trait Predicate: Clone + Send + Sync + 'static {
    /// Evaluate the predicate against the current context. Returning `false`
    /// causes the wrapped handler to skip this cycle.
    fn check(&self, cx: &ExtractCx<'_>) -> bool;
}

impl<F> Predicate for F
where
    F: Fn(&ExtractCx<'_>) -> bool + Clone + Send + Sync + 'static,
{
    fn check(&self, cx: &ExtractCx<'_>) -> bool {
        (self)(cx)
    }
}

/// Handler wrapper produced by [`HandlerExt::run_if`].
///
/// Evaluates the predicate first; if it returns `false`, returns `Ok(())`
/// without invoking the inner handler — so the inner's extractors do not run
/// and no side effects fire.
pub struct RunIf<H, P> {
    handler: H,
    predicate: P,
}

impl<H, P> std::fmt::Debug for RunIf<H, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunIf").finish_non_exhaustive()
    }
}

impl<H: Clone, P: Clone> Clone for RunIf<H, P> {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

impl<H, P, T> Handler<T> for RunIf<H, P>
where
    H: Handler<T>,
    P: Predicate,
    T: Send + 'static,
{
    async fn call(self, cx: &ExtractCx<'_>) -> Result<(), AppError> {
        if !self.predicate.check(cx) {
            return Ok(());
        }
        self.handler.call(cx).await
    }
}

/// Adds `.run_if(predicate)` to every [`Handler`].
pub trait HandlerExt<T>: Handler<T> {
    /// Wrap this handler so it only runs when `predicate(cx)` returns `true`.
    fn run_if<P: Predicate>(self, predicate: P) -> RunIf<Self, P> {
        RunIf {
            handler: self,
            predicate,
        }
    }
}

impl<T, H: Handler<T>> HandlerExt<T> for H {}

// ---------------------------------------------------------------------------
// Standard predicates
// ---------------------------------------------------------------------------

/// Predicate that always returns `true`.
pub fn always() -> impl Predicate {
    |_: &ExtractCx<'_>| true
}

/// Predicate that always returns `false`.
pub fn never() -> impl Predicate {
    |_: &ExtractCx<'_>| false
}

/// Invert a predicate.
pub fn not<P: Predicate>(p: P) -> impl Predicate {
    move |cx: &ExtractCx<'_>| !p.check(cx)
}

/// Conjoin two predicates (short-circuits on the first `false`).
pub fn and<A: Predicate, B: Predicate>(a: A, b: B) -> impl Predicate {
    move |cx: &ExtractCx<'_>| a.check(cx) && b.check(cx)
}

/// Disjoin two predicates (short-circuits on the first `true`).
pub fn or<A: Predicate, B: Predicate>(a: A, b: B) -> impl Predicate {
    move |cx: &ExtractCx<'_>| a.check(cx) || b.check(cx)
}

/// Predicate that extracts a value via [`FromContext`] and runs `check` on
/// it. Returns `false` if the extractor returns `None` (e.g. the resource
/// isn't registered).
///
/// The closure-parameter type infers `E`, so callers don't need a turbofish:
///
/// ```ignore
/// .run_if(in_state(|p: &Phase| p.is_running()))
/// ```
pub fn in_state<E, F>(check: F) -> impl Predicate
where
    E: FromContext + 'static,
    F: Fn(&E) -> bool + Clone + Send + Sync + 'static,
{
    move |cx: &ExtractCx<'_>| match E::from_context(cx) {
        Some(e) => check(&e),
        None => false,
    }
}

// PhantomData kept around for downstream consumers that may rely on the
// crate's existing item list shape even with this `mod` refactor.
#[allow(dead_code)]
struct _ApiAnchor(PhantomData<()>);
