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
//! Predicates are `Fn(&ExtractCx<'_, S>) -> bool` closures — same context
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
/// Any `Fn(&ExtractCx<'_, S>) -> bool` closure that is `Clone + Send + Sync +
/// 'static` automatically implements this trait, so callers write predicates
/// as plain closures and rarely name the trait directly.
pub trait Predicate<S>: Clone + Send + Sync + 'static {
    /// Evaluate the predicate against the current context. Returning `false`
    /// causes the wrapped handler to skip this cycle.
    fn check(&self, cx: &ExtractCx<'_, S>) -> bool;
}

impl<S, F> Predicate<S> for F
where
    F: Fn(&ExtractCx<'_, S>) -> bool + Clone + Send + Sync + 'static,
{
    fn check(&self, cx: &ExtractCx<'_, S>) -> bool {
        (self)(cx)
    }
}

/// Handler wrapper produced by [`HandlerExt::run_if`].
///
/// Evaluates the predicate first; if it returns `false`, returns `Ok(())`
/// without invoking the inner handler — so the inner's extractors do not run
/// and no side effects fire.
pub struct RunIf<H, P, S> {
    handler: H,
    predicate: P,
    _phantom: PhantomData<fn() -> S>,
}

impl<H, P, S> std::fmt::Debug for RunIf<H, P, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunIf").finish_non_exhaustive()
    }
}

impl<H: Clone, P: Clone, S> Clone for RunIf<H, P, S> {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            predicate: self.predicate.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<H, P, S, T> Handler<S, T> for RunIf<H, P, S>
where
    H: Handler<S, T>,
    P: Predicate<S>,
    S: Send + Sync + 'static,
    T: Send + 'static,
{
    async fn call(self, cx: &ExtractCx<'_, S>) -> Result<(), AppError> {
        if !self.predicate.check(cx) {
            return Ok(());
        }
        self.handler.call(cx).await
    }
}

/// Adds `.run_if(predicate)` to every [`Handler`].
pub trait HandlerExt<S, T>: Handler<S, T> {
    /// Wrap this handler so it only runs when `predicate(cx)` returns `true`.
    fn run_if<P: Predicate<S>>(self, predicate: P) -> RunIf<Self, P, S> {
        RunIf {
            handler: self,
            predicate,
            _phantom: PhantomData,
        }
    }
}

impl<S, T, H: Handler<S, T>> HandlerExt<S, T> for H {}

// ---------------------------------------------------------------------------
// Standard predicates
// ---------------------------------------------------------------------------

/// Predicate that always returns `true`.
pub fn always<S>() -> impl Predicate<S> {
    |_: &ExtractCx<'_, S>| true
}

/// Predicate that always returns `false`.
pub fn never<S>() -> impl Predicate<S> {
    |_: &ExtractCx<'_, S>| false
}

/// Invert a predicate.
pub fn not<S, P: Predicate<S>>(p: P) -> impl Predicate<S> {
    move |cx: &ExtractCx<'_, S>| !p.check(cx)
}

/// Conjoin two predicates (short-circuits on the first `false`).
pub fn and<S, A: Predicate<S>, B: Predicate<S>>(a: A, b: B) -> impl Predicate<S> {
    move |cx: &ExtractCx<'_, S>| a.check(cx) && b.check(cx)
}

/// Disjoin two predicates (short-circuits on the first `true`).
pub fn or<S, A: Predicate<S>, B: Predicate<S>>(a: A, b: B) -> impl Predicate<S> {
    move |cx: &ExtractCx<'_, S>| a.check(cx) || b.check(cx)
}

/// Predicate that extracts a value via [`FromContext`] and runs `check` on
/// it. Returns `false` if the extractor returns `None` (e.g. the extension
/// isn't registered).
///
/// The closure-parameter type infers `E`, so callers don't need a turbofish:
///
/// ```ignore
/// .run_if(in_state(|p: &Phase| p.is_running()))
/// ```
pub fn in_state<S, E, F>(check: F) -> impl Predicate<S>
where
    E: FromContext<S> + 'static,
    F: Fn(&E) -> bool + Clone + Send + Sync + 'static,
{
    move |cx: &ExtractCx<'_, S>| match E::from_context(cx) {
        Some(e) => check(&e),
        None => false,
    }
}
