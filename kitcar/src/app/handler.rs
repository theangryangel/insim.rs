//! The magic-extractor `Handler` trait and its arity-N blanket impls.
//!
//! The trait is parameterised over `T` (a tuple of extractor types) and `S`
//! (the app's state type, defaulting to `()`). One blanket impl per arity
//! makes any plain async fn whose arguments all implement [`FromContext<S>`]
//! usable as a handler.
//!
//! Routing-key inference is implicit: if any extractor returns `None` (e.g. the
//! current dispatch isn't the wire variant `Packet<Ncn>` is asking for), the
//! handler is skipped this cycle.
//!
//! ## Stateful handlers
//!
//! Implement `Handler` directly on a struct for private per-registration
//! state. The runtime uniquely owns it and calls it mutably without cloning.
//! State shared between handlers belongs in the application's `State<S>`.

use std::{future::Future, marker::PhantomData};

use futures::future::BoxFuture;

use super::extract::{ExtractCx, FromContext};
use crate::{
    error::AppError,
    ui::{NoView, View},
};

/// A handler: a plain async fn whose parameters are magic extractors, or
/// any struct that owns state and impls this trait manually.
///
/// Implemented for every closure / async-fn with arity 0..=7 whose arguments
/// all implement [`FromContext<S>`]. Implement manually on your own struct
/// for a non-function handler. The default `call` body is a no-op `Ok(())`.
///
/// The trait method is declared with `-> impl Future + Send` (rather than
/// `async fn`) because the runtime hands handlers to a wrapper that requires
/// the future to be `Send`. Implementors can write `async fn call(...)` in
/// their impls - Rust accepts that as long as the body produces a Send
/// future.
pub trait Handler<T, S = (), V = NoView>: Send + Sized + 'static
where
    V: View + 'static,
{
    /// Call the handler mutably against the current dispatch context.
    fn call(
        &mut self,
        _cx: &ExtractCx<'_, S, V>,
    ) -> impl Future<Output = Result<(), AppError>> + Send {
        async { Ok(()) }
    }
}

/// Object-safe shim so handlers with different `T` tuples can live together
/// in the ordered handler map.
pub(crate) trait ErasedHandler<S, V>: Send
where
    V: View + 'static,
{
    fn call<'a>(&'a mut self, cx: &'a ExtractCx<'_, S, V>) -> BoxFuture<'a, Result<(), AppError>>;
}

pub(crate) struct HandlerService<T, H, S> {
    handler: H,
    _phantom: PhantomData<fn(T, S) -> ()>,
}

impl<T, H, S> HandlerService<T, H, S> {
    pub(crate) fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom: PhantomData,
        }
    }
}

impl<T, H, S, V> ErasedHandler<S, V> for HandlerService<T, H, S>
where
    H: Handler<T, S, V> + 'static,
    T: Send + 'static,
    S: Send + Sync + 'static,
    V: View + 'static,
{
    fn call<'a>(&'a mut self, cx: &'a ExtractCx<'_, S, V>) -> BoxFuture<'a, Result<(), AppError>> {
        Box::pin(self.handler.call(cx))
    }
}

macro_rules! impl_handler {
    ( $($ty:ident),* ) => {
        #[allow(non_snake_case)]
        impl<F, Fut, S, V, $($ty),*> Handler<($($ty,)*), S, V> for F
        where
            F: FnMut($($ty),*) -> Fut + Send + 'static,
            $( $ty: FromContext<S, V> + 'static, )*
            Fut: Future<Output = Result<(), AppError>> + Send,
            S: Send + Sync + 'static,
            V: View + 'static,
        {
            #[allow(unused)]
            async fn call(&mut self, cx: &ExtractCx<'_, S, V>) -> Result<(), AppError> {
                // Extract each parameter, returning early if extraction fails
                $(
                    let Some($ty) = $ty::from_context(cx) else {
                        return Ok(());
                    };
                )*

                // Call the function with the extracted variables
                (self)($($ty),*).await
            }
        }
    };
}

impl_handler!();
impl_handler!(T0);
impl_handler!(T0, T1);
impl_handler!(T0, T1, T2);
impl_handler!(T0, T1, T2, T3);
impl_handler!(T0, T1, T2, T3, T4);
impl_handler!(T0, T1, T2, T3, T4, T5);
impl_handler!(T0, T1, T2, T3, T4, T5, T6);
impl_handler!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_handler!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_handler!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_handler!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
