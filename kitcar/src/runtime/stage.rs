//! Your custom logic / stage for Runtime

use std::{fmt::Debug, future::Future};

use futures::future::BoxFuture;

use super::{Context, Error, FromContext, Transition};

/// The type alias for a "stage function".
/// This is now a boxed, type-erased closure that takes a Context
/// and returns a BoxFuture. This allows us to have a cancel safe future.
pub type StageFn<U, E = Error> = Box<
    dyn Fn(Context<U>) -> BoxFuture<'static, std::result::Result<Transition<U, E>, E>>
        + Send
        + Sync
        + 'static,
>;

/// Stage
#[async_trait::async_trait]
pub trait StageHandler<U, E, Args>: Send + Sync + 'static
where
    U: Clone + Debug + Send,
    E: std::error::Error + Send + 'static,
{
    /// Execute/call the Stage
    async fn call(self, ctx: Context<U>) -> std::result::Result<Transition<U, E>, E>;

    /// Helper to package this handler up into a type-erased StateFn.
    fn into_stage_fn(self) -> StageFn<U, E>
    where
        Self: Sized + Clone,
        U: Clone + Debug + Send + 'static,
        Args: 'static,
    {
        let wrapper = move |ctx: Context<U>| {
            let f = self.clone(); // we're not fnonce - because we want to allow re-runs
            let fut = async move {
                // Run handler's "magic" call method. This does all of our extraction magic.
                f.call(ctx).await
            };
            Box::pin(fut) as BoxFuture<'static, std::result::Result<Transition<U, E>, E>>
        };

        Box::new(wrapper)
    }
}

// Base case, no extractors
#[async_trait::async_trait]
impl<U, E, F, Fut> StageHandler<U, E, ()> for F
where
    U: Clone + Debug + Send + 'static,
    E: std::error::Error + Send + 'static,
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = std::result::Result<Transition<U, E>, E>> + Send + 'static,
{
    async fn call(self, _ctx: Context<U>) -> std::result::Result<Transition<U, E>, E> {
        // Just call the function, no extraction needed.
        (self)().await
    }
}

macro_rules! impl_stage_handler {
    // Base case: no extractors (already handled manually)
    () => {};

    // Recursive case: generate impl for N extractors
    ($($E:ident),+) => {
        #[async_trait::async_trait]
        impl<U, E, $($E,)+ F, Fut> StageHandler<U, E, ($($E,)+)> for F
        where
            U: Clone + Debug + Send + 'static,
            E: std::error::Error + Send + 'static,
            $($E: FromContext<U> + Send + 'static,)+
            F: Fn($($E),+) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = std::result::Result<Transition<U, E>, E>> + Send + 'static,
        {
            #[allow(non_snake_case)]
            async fn call(self, ctx: Context<U>) -> std::result::Result<Transition<U, E>, E> {
                // Extract each parameter
                $(
                    let $E = $E::from_context(&ctx)
                        .map_err(|e| {
                            // This is a bit of a hack - we need to convert kitcar::Error to user's error
                            // In practice, if extraction fails, it's a framework error
                            // We'll panic here as it indicates a programming error
                            panic!("Extractor failed: {}", e);
                        })?;
                )+

                // Call the function with all extracted parameters
                (self)($($E),+).await
            }
        }
    };
}

impl_stage_handler!();
impl_stage_handler!(E1);
impl_stage_handler!(E1, E2);
impl_stage_handler!(E1, E2, E3);
impl_stage_handler!(E1, E2, E3, E4);
impl_stage_handler!(E1, E2, E3, E4, E5);
impl_stage_handler!(E1, E2, E3, E4, E5, E6);
impl_stage_handler!(E1, E2, E3, E4, E5, E6, E7);
impl_stage_handler!(E1, E2, E3, E4, E5, E6, E7, E8);
impl_stage_handler!(E1, E2, E3, E4, E5, E6, E7, E8, E9);
impl_stage_handler!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10);
impl_stage_handler!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11);
impl_stage_handler!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12);
impl_stage_handler!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12, E13);
impl_stage_handler!(E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12, E13, E14);
impl_stage_handler!(
    E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12, E13, E14, E15
);
