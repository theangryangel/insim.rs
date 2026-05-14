//! The magic-extractor `Handler` trait and its arity-N blanket impls.
//!
//! The trait is parameterised over `S` (application state) and `T` (a tuple of
//! extractor types). One blanket impl per arity makes any plain async fn whose
//! arguments all implement [`FromContext<S>`] usable as a handler.
//!
//! Routing key inference is implicit: if any extractor returns `None` (e.g. the
//! current dispatch isn't the wire variant `Packet<Ncn>` is asking for), the
//! handler is skipped this cycle.

use std::{future::Future, marker::PhantomData};

use futures::future::BoxFuture;

use crate::{
    error::AppError,
    extract::{ExtractCx, FromContext},
};

/// A handler: a plain async fn whose parameters are magic extractors.
///
/// Implemented for every closure / async-fn with arity 1..=4 whose arguments
/// all implement [`FromContext<S>`]. Implement manually on your own struct for
/// a non-function handler.
///
/// The trait method is declared with `-> impl Future + Send` (rather than
/// `async fn`) because the runtime hands handlers to a wrapper that requires
/// the future to be `Send`. Implementors can still write `async fn call(...)`
/// in their `impl`s - Rust accepts that as long as the body produces a Send
/// future.
pub trait Handler<S, T>: Clone + Send + Sized + 'static {
    /// Call the handler against the current dispatch context.
    fn call(self, cx: &ExtractCx<'_, S>) -> impl Future<Output = Result<(), AppError>> + Send;
}

/// Object-safe shim so handlers with different `T` tuples can live together in a `Vec`.
pub(crate) trait ErasedHandler<S>: Send {
    fn call<'a>(&'a self, cx: &'a ExtractCx<'_, S>) -> BoxFuture<'a, Result<(), AppError>>;
}

pub(crate) struct HandlerService<S, T, H> {
    handler: H,
    _phantom: PhantomData<fn(S, T) -> ()>,
}

impl<S, T, H> HandlerService<S, T, H> {
    pub(crate) fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom: PhantomData,
        }
    }
}

impl<S, T, H> ErasedHandler<S> for HandlerService<S, T, H>
where
    H: Handler<S, T> + 'static,
    S: Send + Sync + 'static,
    T: Send + 'static,
{
    fn call<'a>(&'a self, cx: &'a ExtractCx<'_, S>) -> BoxFuture<'a, Result<(), AppError>> {
        let h = self.handler.clone();
        Box::pin(async move { h.call(cx).await })
    }
}

macro_rules! impl_handler {
    ( $($ty:ident),* ) => {
        #[allow(non_snake_case)]
        impl<S, F, Fut, $($ty),*> Handler<S, ($($ty,)*)> for F
        where
            F: FnOnce($($ty),*) -> Fut + Clone + Send + 'static,
            $( $ty: FromContext<S> + 'static, )*
            Fut: Future<Output = Result<(), AppError>> + Send,
            S: Send + Sync + 'static,
        {
            #[allow(unused)]
            async fn call(self, cx: &ExtractCx<'_, S>) -> Result<(), AppError> {
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
