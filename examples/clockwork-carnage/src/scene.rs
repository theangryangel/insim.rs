use std::marker::PhantomData;

// A stage/layer/scene that orchestrates game flow. long live, delegates to other scene in a
// waterfall manner
/// Scene can succeed, bail (stop chain without error), or error
pub trait Scene {
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;

    async fn run(self) -> Result<SceneResult<Self::Output>, Self::Error>
    where
        Self: Sized;
}

#[derive(Debug)]
pub enum SceneResult<T> {
    /// Continue to next scene with this value
    Continue(T),
    /// Stop the chain gracefully (not an error), and allow a repeat
    Bail,
    #[allow(unused)]
    /// Quit stops the chain and does not allow a repeat
    Quit,
}

/// Kind of SceneError
#[derive(Debug, thiserror::Error)]
pub enum SceneErrorKind {
    #[error("Insim error: {0}")]
    Insim(#[from] insim::Error),

    // Scene specific error
    #[error("{scene}: {cause}")]
    Scene {
        scene: &'static str,
        #[source]
        cause: Box<dyn std::error::Error + Send + Sync>
    }
}

/// Scene Error
// FIXME: Drop associated Error type from Scene trait. 
// everything should return this, and impl it's own conversions
// Add a WithContext trait to allow context("asdasd").. to both SceneError and SceneErrorKind
#[derive(Debug)]
pub struct SceneError {
    kind: SceneErrorKind,
    context: String,
}

impl std::fmt::Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)?;
        if !self.context.is_empty() {
            write!(f, ": {}", self.context)?;
        }
        Ok(())
    }
}

impl std::error::Error for SceneError {}

// Scene Combinators - do this then...
// No data is passed between Scenes. If you need this with ThenWith.
#[derive(Debug, Clone)]
pub struct Then<A, B> {
    first: A,
    second: B,
}

impl<A, B> Scene for Then<A, B>
where
    A: Scene + Send + 'static,
    B: Scene + Send + 'static,
    A::Error: Into<B::Error>,
    A::Output: Send + 'static,
    B::Output: Send + 'static,
{
    type Output = B::Output;
    type Error = B::Error;

    async fn run(self) -> Result<SceneResult<Self::Output>, Self::Error> {
        match self.first.run().await.map_err(Into::into)? {
            SceneResult::Continue(_) => self.second.run().await,
            SceneResult::Bail => Ok(SceneResult::Bail),
            SceneResult::Quit => Ok(SceneResult::Quit),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThenWith<A, B, F> {
    first: A,
    next_fn: F,
    _phantom: PhantomData<B>,
}

impl<A, B, F> Scene for ThenWith<A, B, F>
where 
    A: Scene + Send + 'static,
    B: Scene + Send + 'static,
    A::Error: Into<B::Error>,
    F: Fn(A::Output) -> B + Clone,
{
    type Output = B::Output;
    type Error = B::Error;

    async fn run(self) -> Result<SceneResult<Self::Output>, Self::Error>
    where
        Self: Sized {
        match self.first.run().await.map_err(Into::into)? {
            SceneResult::Continue(res) => {
                let second = (self.next_fn)(res);
                second.run().await
            },
            SceneResult::Bail => Ok(SceneResult::Bail),
            SceneResult::Quit => Ok(SceneResult::Quit),
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
    type Error = S::Error;

    async fn run(self) -> Result<SceneResult<()>, Self::Error> {
        loop {
            match self.scene.clone().run().await? {
                SceneResult::Continue(_) => continue,
                SceneResult::Bail => continue,
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

    fn then_with<S, F>(self, f: F) -> ThenWith<Self, S, F>
    where
        S: Scene,
        F: Fn(Self::Output) -> S + Clone
    {
        ThenWith {
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
}

// Blanket impl
impl<T: Scene> SceneExt for T {}
