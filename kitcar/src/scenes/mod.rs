//! Orchestrate layers/scenes of a game. Each scene is long lived and delegates in a waterfall
//! manner and can optionally automatically recover.
use std::{marker::PhantomData, time::Duration};

pub mod wait_for_players;

/// Extract a value from a shared context. Implement this for each infrastructure type you want
/// scenes to receive automatically (axum-style extractor pattern).
pub trait FromContext<Ctx>: Sized + Clone {
    /// Extract `Self` from the given context.
    fn from_context(ctx: &Ctx) -> Self;
}

/// A stage/layer/scene that orchestrates game flow. long live, delegates to other scene in a
/// waterfall manner
/// Scene can succeed, bail (stop chain without error), or error
pub trait Scene<Ctx = ()> {
    /// Output from the scene, may be passed to subsequence scenes through the AndThen combinator
    type Output;

    /// Run/execute the scene
    #[allow(async_fn_in_trait)]
    async fn run(self, ctx: &Ctx) -> Result<SceneResult<Self::Output>, SceneError>
    where
        Self: Sized;
}

#[derive(Debug)]
/// Scene Result
pub enum SceneResult<T> {
    /// Continue to next scene with this value
    Continue(T),
    /// Stop the chain gracefully (not an error), and allow a repeat
    Bail {
        /// Optional reason for bailing
        msg: Option<String>,
    },
    #[allow(unused)]
    /// Quit stops the chain and does not allow a repeat
    Quit,
}

impl<T> SceneResult<T> {
    /// Shortcut to make SceneResult::Bail
    #[allow(unused)]
    pub fn bail() -> Self {
        Self::Bail { msg: None }
    }

    /// Shortcut to make SceneResult::Bail with a reason/msg
    pub fn bail_with(msg: impl Into<String>) -> Self {
        Self::Bail {
            msg: Some(msg.into()),
        }
    }
}

/// Kind of SceneError
#[derive(Debug, thiserror::Error)]
pub enum SceneError {
    /// Insim
    #[error("Insim error: {0}")]
    Insim(#[from] insim::Error),

    /// Lost Insim handle
    #[error("Insim handle lost")]
    InsimHandleLost,

    /// Custom error
    #[error("{scene}: {cause}")]
    #[allow(unused)]
    Custom {
        /// Origin
        scene: &'static str,
        #[source]
        /// Cause
        cause: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Scene Combinators - do this then...
/// No data is passed between Scenes. If you need data passed use AndThen
#[derive(Debug, Clone)]
pub struct Then<A, B> {
    first: A,
    second: B,
}

impl<A, B, Ctx> Scene<Ctx> for Then<A, B>
where
    A: Scene<Ctx> + Send + 'static,
    B: Scene<Ctx> + Send + 'static,
    A::Output: Send + 'static,
    B::Output: Send + 'static,
    Ctx: Sync,
{
    type Output = B::Output;

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<Self::Output>, SceneError> {
        match self.first.run(ctx).await? {
            SceneResult::Continue(_) => self.second.run(ctx).await,
            SceneResult::Bail { msg } => Ok(SceneResult::Bail { msg }),
            SceneResult::Quit => Ok(SceneResult::Quit),
        }
    }
}

/// Scene Combinators - do this then using a closure to create the next scene. The output from the
/// previous scene is passed as the argument
#[derive(Debug, Clone)]
pub struct AndThen<A, B, F> {
    first: A,
    next_fn: F,
    _phantom: PhantomData<B>,
}

impl<A, B, F, Ctx> Scene<Ctx> for AndThen<A, B, F>
where
    A: Scene<Ctx> + Send + 'static,
    B: Scene<Ctx> + Send + 'static,
    F: Fn(A::Output) -> B + Clone,
    Ctx: Sync,
{
    type Output = B::Output;

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<Self::Output>, SceneError>
    where
        Self: Sized,
    {
        match self.first.run(ctx).await? {
            SceneResult::Continue(res) => {
                let second = (self.next_fn)(res);
                second.run(ctx).await
            },
            SceneResult::Bail { msg } => Ok(SceneResult::Bail { msg }),
            SceneResult::Quit => Ok(SceneResult::Quit),
        }
    }
}

/// Wrap a scene with a timeout
#[derive(Debug, Clone)]
pub struct WithTimeout<S> {
    inner: S,
    timeout: Duration,
}

impl<S, Ctx> Scene<Ctx> for WithTimeout<S>
where
    S: Scene<Ctx> + Send + 'static,
    S::Output: Send + 'static,
    Ctx: Sync,
{
    type Output = S::Output;

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<Self::Output>, SceneError> {
        match tokio::time::timeout(self.timeout, self.inner.run(ctx)).await {
            Ok(result) => result,
            Err(_) => Ok(SceneResult::bail_with("WithTimeout")),
        }
    }
}

/// Scene Combinators - repeat the chain on bail
pub struct LoopUntilQuit<S> {
    scene: S,
}

impl<S, Ctx> Scene<Ctx> for LoopUntilQuit<S>
where
    S: Scene<Ctx> + Clone + Send + 'static,
    S::Output: Send + 'static,
    Ctx: Sync,
{
    type Output = ();

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<()>, SceneError> {
        loop {
            match self.scene.clone().run(ctx).await? {
                SceneResult::Continue(_) => continue,
                SceneResult::Bail { msg } => {
                    tracing::info!("Bailed, restarting: {msg:?}");
                    continue;
                },
                SceneResult::Quit => return Ok(SceneResult::Quit),
            }
        }
    }
}

/// Helper trait for constructing scene combinator chains.
pub trait SceneExt: Sized {
    /// Shortcut to use the Then combinators
    fn then<S>(self, next: S) -> Then<Self, S> {
        Then {
            first: self,
            second: next,
        }
    }

    /// When a scene bails, automatically start back at the beginning
    fn loop_until_quit(self) -> LoopUntilQuit<Self>
    where
        Self: Clone,
    {
        LoopUntilQuit { scene: self }
    }

    /// Shortcut to use the Timeout combinator
    fn with_timeout(self, timeout: Duration) -> WithTimeout<Self>
    where
        Self: Clone,
    {
        WithTimeout {
            inner: self,
            timeout,
        }
    }
}

// Blanket impl for all types
impl<T> SceneExt for T {}
