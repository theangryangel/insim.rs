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

// Scene Combinators - do this then...
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

    fn repeat(self) -> Repeat<Self>
    where
        Self: Clone,
    {
        Repeat { scene: self }
    }
}

// Blanket impl
impl<T: Scene> SceneExt for T {}
