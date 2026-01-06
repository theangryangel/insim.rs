// A stage/layer/scene that orchestrates game flow. long live, delegates to other scene in a
// waterfall manner
pub trait Scene<C>: Send + Sync + 'static {
    type Output: std::fmt::Debug + Send + Sync + 'static;
    fn poll(&mut self, ctx: C) -> impl Future<Output = Self::Output> + Send;
}
