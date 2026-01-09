use std::{marker::PhantomData, time::Duration};

// A stage/layer/scene that orchestrates game flow. long live, delegates to other scene in a
// waterfall manner
/// Scene can succeed, bail (stop chain without error), or error
pub trait Scene {
    type Output;

    async fn run(self) -> Result<SceneResult<Self::Output>, SceneError>
    where
        Self: Sized;
}

#[derive(Debug)]
pub enum SceneResult<T> {
    /// Continue to next scene with this value
    Continue(T),
    /// Stop the chain gracefully (not an error), and allow a repeat
    Bail { msg: Option<String> },
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

    #[error("Insim handle lost")]
    InsimHandleLost,

    #[error("Chat handle lost")]
    ChatHandleLost,

    /// Custom error
    #[error("{scene}: {cause}")]
    #[allow(unused)]
    Custom {
        scene: &'static str,
        #[source]
        cause: Box<dyn std::error::Error + Send + Sync>,
    },
}

// Scene Combinators - do this then...
// No data is passed between Scenes. If you need this with AndThen
#[derive(Debug, Clone)]
pub struct Then<A, B> {
    first: A,
    second: B,
}

impl<A, B> Scene for Then<A, B>
where
    A: Scene + Send + 'static,
    B: Scene + Send + 'static,
    A::Output: Send + 'static,
    B::Output: Send + 'static,
{
    type Output = B::Output;

    async fn run(self) -> Result<SceneResult<Self::Output>, SceneError> {
        match self.first.run().await? {
            SceneResult::Continue(_) => self.second.run().await,
            SceneResult::Bail { msg } => Ok(SceneResult::Bail { msg }),
            SceneResult::Quit => Ok(SceneResult::Quit),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AndThen<A, B, F> {
    first: A,
    next_fn: F,
    _phantom: PhantomData<B>,
}

impl<A, B, F> Scene for AndThen<A, B, F>
where
    A: Scene + Send + 'static,
    B: Scene + Send + 'static,
    F: Fn(A::Output) -> B + Clone,
{
    type Output = B::Output;

    async fn run(self) -> Result<SceneResult<Self::Output>, SceneError>
    where
        Self: Sized,
    {
        match self.first.run().await? {
            SceneResult::Continue(res) => {
                let second = (self.next_fn)(res);
                second.run().await
            },
            SceneResult::Bail { msg } => Ok(SceneResult::Bail { msg }),
            SceneResult::Quit => Ok(SceneResult::Quit),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WithTimeout<S> {
    inner: S,
    timeout: Duration,
}

impl<S> Scene for WithTimeout<S>
where
    S: Scene + Send + 'static,
    S::Output: Send + 'static,
{
    type Output = S::Output;

    async fn run(self) -> Result<SceneResult<Self::Output>, SceneError> {
        match tokio::time::timeout(self.timeout, self.inner.run()).await {
            Ok(result) => result,
            Err(_) => Ok(SceneResult::bail_with("WithTimeout")),
        }
    }
}

// Scene Combinators - repeat the chain on bail
pub struct Repeat<S> {
    scene: S,
}

impl<S> Scene for Repeat<S>
where
    S: Scene + Clone + Send + 'static,
    S::Output: Send + 'static,
{
    type Output = ();

    async fn run(self) -> Result<SceneResult<()>, SceneError> {
        loop {
            match self.scene.clone().run().await? {
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

// Helper trait
pub trait SceneExt: Scene + Sized {
    fn then<S>(self, next: S) -> Then<Self, S>
    where
        S: Scene,
    {
        Then {
            first: self,
            second: next,
        }
    }

    fn and_then<S, F>(self, f: F) -> AndThen<Self, S, F>
    where
        S: Scene,
        F: Fn(Self::Output) -> S + Clone,
    {
        AndThen {
            first: self,
            next_fn: f,
            _phantom: PhantomData,
        }
    }

    fn repeat(self) -> Repeat<Self>
    where
        Self: Clone,
    {
        Repeat { scene: self }
    }

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

// Blanket impl
impl<T: Scene> SceneExt for T {}
