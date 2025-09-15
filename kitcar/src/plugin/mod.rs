//! Plugins.

pub mod context;

use std::{fmt::Debug, future::Future};

pub use context::PluginContext;

/// User State
pub trait UserState: Send + Sync + Clone + Debug + 'static {}
impl UserState for () {}

/// Plugin trait
#[async_trait::async_trait]
pub trait Plugin<S>: Send + Sync
where
    S: UserState,
{
    /// Run
    // FIXME: some kind of error?
    async fn run(mut self: Box<Self>, ctx: PluginContext<S>) -> Result<(), ()>;
}

/// Allow the user to do something like this:
/// pub async fn chatterbox(mut ctx: TaskContext<State>) {
///     info!("hello world");
/// }
///
/// framework.with_plugin(chatter_box);
#[async_trait::async_trait]
impl<S, F, Fut> Plugin<S> for F
where
    S: UserState,
    F: Fn(PluginContext<S>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), ()>> + Send + 'static,
{
    async fn run(self: Box<Self>, ctx: PluginContext<S>) -> Result<(), ()> {
        (self)(ctx).await
    }
}
