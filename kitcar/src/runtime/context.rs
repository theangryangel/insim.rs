//! Context for Runtime

use std::fmt::Debug;

use super::Result;

/// Extractor for custom user state
#[allow(missing_debug_implementations)]
pub struct State<U>(pub U);

/// The `Context` provides the API for state functions to interact with
/// the game world.
///
/// It is `Clone` and `Send + Sync` so it can be passed into any `async` task.
/// It's empty for now, but will hold shared state, event senders, etc.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Context<U: Clone + Debug> {
    /// Custom user state
    pub state: U,
    /// Insim
    pub insim: insim::builder::SpawnedHandle,
}

impl<U: Clone + Debug> Context<U> {
    /// Creates a new, empty context.
    pub fn new(insim: insim::builder::SpawnedHandle, user: U) -> Self {
        Context { state: user, insim }
    }
}

/// Trait for "extracting" values from the game Context.
pub trait FromContext<U: Clone + Debug + Send>: Sized {
    /// Perform the extraction.
    fn from_context(ctx: &Context<U>) -> Result<Self>;
}

impl<U: Clone + Debug + Send> FromContext<U> for State<U> {
    fn from_context(ctx: &Context<U>) -> Result<Self> {
        Ok(State(ctx.state.clone()))
    }
}

impl<U: Clone + Debug + Send> FromContext<U> for insim::builder::SpawnedHandle {
    fn from_context(ctx: &Context<U>) -> Result<Self> {
        Ok(ctx.insim.clone())
    }
}
