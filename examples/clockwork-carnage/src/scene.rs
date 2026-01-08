use std::{error::Error, fmt::Debug};

// A stage/layer/scene that orchestrates game flow. long live, delegates to other scene in a
// waterfall manner
pub trait Scene<C>: Send + Sync + 'static {
    type Output: std::fmt::Debug + Send + Sync + 'static;
    type Error: SceneError;
    fn poll(&mut self, ctx: C) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}

pub trait SceneError: Error + Debug + Send + Sync + 'static {
    fn is_recoverable(&self) -> bool;
}
