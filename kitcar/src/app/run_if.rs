//! Conditional handler gating via `.run_if(predicate)`.
//!
//! [`HandlerExt::run_if`] accepts any value implementing [`RunIf<T, S>`].
//! Blanket impls cover `Fn(E0, …) -> bool` closures for arities 0–4 when the
//! handler itself pins `S` (e.g. via a `State<S>` argument).
//!
//! ```ignore
//! // Works when handler already constrains S (e.g. has State<Bomb> arg):
//! handler.run_if(|s: State<Bomb>| s.read().phase == Phase::Racing)
//! ```
//!
//! If any extractor returns `None` the predicate short-circuits to `false` and
//! the inner handler is skipped without running its own extractors.

use std::marker::PhantomData;

use super::{
    extract::{ExtractCx, FromContext},
    handler::Handler,
};
use crate::error::AppError;

/// A synchronous predicate over the dispatch context.
///
/// Implemented automatically for `Fn(E0, E1, …) -> bool` closures (arities
/// 0–4) whose arguments all implement [`FromContext<S>`]. Implement manually
/// for custom predicate types.
pub trait RunIf<T, S = ()>: Clone + Send + Sync + 'static {
    /// Evaluate the predicate. Returns `false` if any extractor returns `None`.
    fn check(&self, cx: &ExtractCx<'_, S>) -> bool;
}

/// Blanket impls for `Fn(…) -> bool` closures.
///
/// These work when the closure's argument types constrain `S` to a single
/// concrete type (e.g. `State<Bomb>` forces `S = Bomb`). If `S` remains
/// ambiguous at the call site, add a `_: State<S>` argument to the closure to
/// pin inference.
macro_rules! impl_run_if {
    ( $($ty:ident),* ) => {
        #[allow(non_snake_case)]
        impl<F, S, $($ty),*> RunIf<($($ty,)*), S> for F
        where
            F: Fn($($ty),*) -> bool + Clone + Send + Sync + 'static,
            $( $ty: FromContext<S> + 'static, )*
            S: Send + Sync + 'static,
        {
            #[allow(unused)]
            fn check(&self, cx: &ExtractCx<'_, S>) -> bool {
                $(
                    let Some($ty) = $ty::from_context(cx) else {
                        return false;
                    };
                )*
                (self)($($ty),*)
            }
        }
    };
}

impl_run_if!();
impl_run_if!(T0);
impl_run_if!(T0, T1);
impl_run_if!(T0, T1, T2);
impl_run_if!(T0, T1, T2, T3);
impl_run_if!(T0, T1, T2, T3, T4);
impl_run_if!(T0, T1, T2, T3, T4, T5);
impl_run_if!(T0, T1, T2, T3, T4, T5, T6);
impl_run_if!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_run_if!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_run_if!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_run_if!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);

/// Handler wrapper produced by [`HandlerExt::run_if`].
///
/// Evaluates the predicate first; if it returns `false`, returns `Ok(())`
/// without invoking the inner handler - so the inner's extractors do not run
/// and no side effects fire.
pub struct Conditional<H, P, PT> {
    handler: H,
    predicate: P,
    _phantom: PhantomData<fn(PT)>,
}

impl<H, P, PT> std::fmt::Debug for Conditional<H, P, PT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Conditional").finish_non_exhaustive()
    }
}

impl<H: Clone, P: Clone, PT> Clone for Conditional<H, P, PT> {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            predicate: self.predicate.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<H, P, HT, PT, S> Handler<HT, S> for Conditional<H, P, PT>
where
    H: Handler<HT, S>,
    P: RunIf<PT, S>,
    HT: Send + 'static,
    PT: Send + 'static,
    S: Send + Sync + 'static,
{
    async fn call(self, cx: &ExtractCx<'_, S>) -> Result<(), AppError> {
        if !self.predicate.check(cx) {
            return Ok(());
        }
        self.handler.call(cx).await
    }
}

/// Adds `.run_if(predicate)` to every [`Handler`].
pub trait HandlerExt<T, S = ()>: Handler<T, S> {
    /// Wrap this handler so it only runs when `predicate` returns `true`.
    fn run_if<P, PT>(self, predicate: P) -> Conditional<Self, P, PT>
    where
        P: RunIf<PT, S>,
        PT: Send + 'static,
    {
        Conditional {
            handler: self,
            predicate,
            _phantom: PhantomData,
        }
    }
}

impl<T, S, H: Handler<T, S>> HandlerExt<T, S> for H {}
