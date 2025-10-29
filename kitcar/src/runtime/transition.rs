//! Transition between Stages

use std::fmt::Debug;

use super::{Error, StageFn, StageHandler};

/// Tells the `Chassis` what to do after a state function finishes.
#[allow(missing_debug_implementations)]
pub enum Transition<U, E = Error>
where
    U: Clone + Debug + Send,
    E: std::error::Error + Send + 'static,
{
    /// Spawn a new state function, replacing the current one.
    /// This now holds our boxed StageFn.
    Next(StageFn<U, E>),
    /// Rerun - workaround for compiler recursion issues
    Again,
    /// Exit the loop entirely.
    Exit,
}

impl<U, E> Transition<U, E>
where
    U: Clone + Debug + Send + 'static,
    E: std::error::Error + Send + 'static,
{
    /// Helper function for users to easily transition to a new state.
    ///
    /// This is now generic: it accepts any `async fn(Context) -> NextState`
    /// and automatically boxes it for the game loop.
    pub fn next<Args, H>(handler: H) -> Self
    where
        H: StageHandler<U, E, Args> + Clone, // Clone is needed to move it into the 'static closure
        Args: 'static,
    {
        Transition::Next(handler.into_stage_fn())
    }

    /// Helper function for users to easily exit the game.
    /// e.g., `return NextState::exit();`
    pub fn exit() -> Self {
        Transition::Exit
    }
}
