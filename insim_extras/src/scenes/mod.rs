//! Orchestrate layers/scenes of a game. Each scene is long lived and delegates in a waterfall
//! manner and can optionally automatically recover.
use std::{marker::PhantomData, time::Duration};

pub use tokio_util::sync::CancellationToken;

use crate::game;

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
    // async_fn_in_trait is stable since Rust 1.75. The lint is suppressed because
    // the returned future is not required to be Send, which is intentional here -
    // scene combinators propagate Send bounds at the impl level.
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
            Err(_) => Ok(SceneResult::bail_with("scene timed out")),
        }
    }
}

/// Wrap a scene so that it exits with [`SceneResult::Quit`] when a [`CancellationToken`] is
/// triggered.
///
/// Place this as the *outermost* combinator so that inner retry loops (e.g. [`LoopUntilQuit`])
/// are themselves cancelled rather than restarted. When the token fires, the inner future is
/// dropped at the next `await` point - no teardown is implicit here.
///
/// ```text
/// game_scene
///     .loop_until_quit()
///     .with_cancellation(token)  // ← outermost: kills the loop when signalled
/// ```
#[derive(Debug, Clone)]
pub struct WithCancellation<S> {
    inner: S,
    token: CancellationToken,
}

impl<S, Ctx> Scene<Ctx> for WithCancellation<S>
where
    S: Scene<Ctx> + Send + 'static,
    S::Output: Send + 'static,
    Ctx: Sync,
{
    type Output = S::Output;

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<Self::Output>, SceneError> {
        tokio::select! {
            biased;
            _ = self.token.cancelled() => Ok(SceneResult::Quit),
            result = self.inner.run(ctx) => result,
        }
    }
}

/// Scene Combinators - repeat the chain on bail
#[derive(Debug, Clone)]
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

/// Wrap a scene so that it exits with [`SceneResult::Continue`]`(())` when the game ends.
///
/// Races the inner scene against [`game::Game::wait_for_end`]. Whichever fires first:
/// - Game ends first → `Continue(())`, allowing an outer [`LoopUntilQuit`] to restart.
/// - Inner scene returns `Continue(_)` → `Continue(())`.
/// - Inner scene returns `Bail` or `Quit` → propagated as-is.
///
/// Place this around the long-running game loop scene, inside `loop_until_quit`:
///
/// ```text
/// WaitForPlayers
///     .then(SetupTrack)
///     .then(ChallengeLoop.until_game_ends())  // ← game lifecycle handled here
///     .loop_until_quit()
///     .with_cancellation(token)
/// ```
#[derive(Debug, Clone)]
pub struct UntilGameEnds<S> {
    inner: S,
}

impl<S, Ctx> Scene<Ctx> for UntilGameEnds<S>
where
    S: Scene<Ctx> + Send + 'static,
    S::Output: Send + 'static,
    game::Game: FromContext<Ctx>,
    Ctx: Sync,
{
    type Output = ();

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<()>, SceneError> {
        let game = game::Game::from_context(ctx);
        tokio::select! {
            biased;
            result = self.inner.run(ctx) => match result? {
                SceneResult::Continue(_) => Ok(SceneResult::Continue(())),
                SceneResult::Bail { msg } => Ok(SceneResult::Bail { msg }),
                SceneResult::Quit => Ok(SceneResult::Quit),
            },
            _ = async { let _ = game.wait_for_end().await; } => {
                Ok(SceneResult::Continue(()))
            },
        }
    }
}

/// Helper trait for constructing scene combinator chains.
///
/// Implemented for every `T: Scene<Ctx>`. The `Ctx` type parameter is almost always inferred
/// from the eventual `.run(ctx)` call, so call sites read naturally:
///
/// ```text
/// scene_a
///     .and_then(|output| SceneB::from(output))
///     .then(SceneC)
///     .loop_until_quit()
///     .with_cancellation(token)
/// ```
pub trait SceneExt<Ctx>: Scene<Ctx> + Sized {
    /// Run this scene, then `next` if it returned [`SceneResult::Continue`].
    ///
    /// The output of this scene is discarded. Use [`SceneExt::and_then`] if the next scene
    /// needs to be constructed from this scene's output.
    fn then<S>(self, next: S) -> Then<Self, S> {
        Then {
            first: self,
            second: next,
        }
    }

    /// Run this scene, then use its output to construct and run the next scene.
    ///
    /// Unlike [`SceneExt::then`], the closure receives `Self::Output` and can build the next
    /// scene dynamically - the only way to pass runtime data between scenes.
    fn and_then<B, F>(self, f: F) -> AndThen<Self, B, F>
    where
        F: Fn(Self::Output) -> B,
    {
        AndThen {
            first: self,
            next_fn: f,
            _phantom: PhantomData,
        }
    }

    /// Restart the scene from the beginning on [`SceneResult::Bail`] or [`SceneResult::Continue`];
    /// stop only on [`SceneResult::Quit`].
    fn loop_until_quit(self) -> LoopUntilQuit<Self>
    where
        Self: Clone,
    {
        LoopUntilQuit { scene: self }
    }

    /// Bail if the scene does not complete within `timeout`.
    fn with_timeout(self, timeout: Duration) -> WithTimeout<Self> {
        WithTimeout {
            inner: self,
            timeout,
        }
    }

    /// Quit the scene immediately if `token` is cancelled, dropping the inner future at the next
    /// `await` point.
    fn with_cancellation(self, token: CancellationToken) -> WithCancellation<Self> {
        WithCancellation { inner: self, token }
    }

    /// Exit with `Continue(())` when the game ends, dropping the inner future at the next
    /// `await` point. See [`UntilGameEnds`] for placement guidance.
    fn until_game_ends(self) -> UntilGameEnds<Self> {
        UntilGameEnds { inner: self }
    }
}

impl<T: Scene<Ctx>, Ctx> SceneExt<Ctx> for T {}

/// Extension trait that adds `.scene_err(scene_name)` to any `Result`.
///
/// Converts a generic error into [`SceneError::Custom`] with a static scene label, eliminating
/// the multi-line `map_err(|cause| SceneError::Custom { scene, cause: Box::new(cause) })` pattern.
///
/// # Example
/// ```ignore
/// use insim_extras::scenes::IntoSceneError as _;
///
/// presence.spec(ucid).await.scene_err("my_game::tick::spec")?;
/// ```
pub trait IntoSceneError<T> {
    /// Wrap the error as [`SceneError::Custom`] labelled with `scene`.
    fn scene_err(self, scene: &'static str) -> Result<T, SceneError>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> IntoSceneError<T> for Result<T, E> {
    fn scene_err(self, scene: &'static str) -> Result<T, SceneError> {
        self.map_err(|cause| SceneError::Custom {
            scene,
            cause: Box::new(cause),
        })
    }
}
