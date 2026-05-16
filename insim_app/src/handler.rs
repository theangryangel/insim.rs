//! The magic-extractor `Handler` trait and its arity-N blanket impls.
//!
//! The trait is parameterised over `T` (a tuple of extractor types). One
//! blanket impl per arity makes any plain async fn whose arguments all
//! implement [`FromContext`] usable as a handler.
//!
//! Routing-key inference is implicit: if any extractor returns `None` (e.g. the
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
/// Implemented for every closure / async-fn with arity 0..=7 whose arguments
/// all implement [`FromContext`]. Implement manually on your own struct for a
/// non-function handler.
///
/// The trait method is declared with `-> impl Future + Send` (rather than
/// `async fn`) because the runtime hands handlers to a wrapper that requires
/// the future to be `Send`. Implementors can still write `async fn call(...)`
/// in their `impl`s - Rust accepts that as long as the body produces a Send
/// future.
pub trait Handler<T>: Clone + Send + Sized + 'static {
    /// Call the handler against the current dispatch context.
    fn call(self, cx: &ExtractCx<'_>) -> impl Future<Output = Result<(), AppError>> + Send;
}

/// Object-safe shim so handlers with different `T` tuples can live together in a `Vec`.
pub(crate) trait ErasedHandler: Send {
    fn call<'a>(&'a self, cx: &'a ExtractCx<'_>) -> BoxFuture<'a, Result<(), AppError>>;
}

pub(crate) struct HandlerService<T, H> {
    handler: H,
    _phantom: PhantomData<fn(T) -> ()>,
}

impl<T, H> HandlerService<T, H> {
    pub(crate) fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom: PhantomData,
        }
    }
}

impl<T, H> ErasedHandler for HandlerService<T, H>
where
    H: Handler<T> + 'static,
    T: Send + 'static,
{
    fn call<'a>(&'a self, cx: &'a ExtractCx<'_>) -> BoxFuture<'a, Result<(), AppError>> {
        let h = self.handler.clone();
        Box::pin(async move { h.call(cx).await })
    }
}

macro_rules! impl_handler {
    ( $($ty:ident),* ) => {
        #[allow(non_snake_case)]
        impl<F, Fut, $($ty),*> Handler<($($ty,)*)> for F
        where
            F: FnOnce($($ty),*) -> Fut + Clone + Send + 'static,
            $( $ty: FromContext + 'static, )*
            Fut: Future<Output = Result<(), AppError>> + Send,
        {
            #[allow(unused)]
            async fn call(self, cx: &ExtractCx<'_>) -> Result<(), AppError> {
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
