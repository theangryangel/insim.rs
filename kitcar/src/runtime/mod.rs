//! Runtime provides a high level state machine and primitives to coordinate the state of a
//! mini-game

use std::{any::Any, fmt::Debug, time::Duration};

use futures::future::BoxFuture;
use tokio::{task::JoinHandle, time::interval};

pub mod context;
pub mod result;
pub mod stage;
pub mod transition;

pub use context::{Context, FromContext};
pub use result::{Error, Result};
pub use stage::{StageFn, StageHandler};
pub use transition::Transition;

/// The runtime is the framework that ties insim, systems, and your custom phases together.
#[allow(missing_debug_implementations)]
pub struct Runtime<U: Clone + Debug + Send> {
    context: Context<U>,
}

impl<U: Clone + Debug + Send> Runtime<U> {
    /// New!
    pub fn new(insim: insim::builder::SpawnedHandle, state: U) -> Self {
        Self {
            context: Context::new(insim, state),
        }
    }

    /// Ignite the state machine
    pub fn ignite<Args, H, E>(
        self,
        initial_state: H,
    ) -> BoxFuture<'static, std::result::Result<(), E>>
    where
        E: std::error::Error + Send + 'static,
        U: Clone + Debug + Send + 'static,
        H: StageHandler<U, E, Args> + Clone,
        Args: 'static,
    {
        // Spawn the state function as a new task.
        let mut current_fn = initial_state.into_stage_fn();
        let mut state_task = (current_fn)(self.context.clone());

        let mut i = interval(Duration::from_secs(1));

        // We return a `BoxFuture` that contains the async logic.
        // `BoxFuture` is `Unpin`, so this can be used in `select!` safely. Thus avoiding the need
        // for the user to manually pin it.
        Box::pin(async move {
            println!("[GameLoop::run] Starting the game loop...");

            loop {
                tokio::select! {
                    result = &mut state_task => {
                        match result {
                            Ok(Transition::Next(next_fn)) => {
                                current_fn = next_fn;
                                state_task = current_fn(self.context.clone());
                            },
                            Ok(Transition::Again) => {
                                state_task = current_fn(self.context.clone());
                            },
                            Ok(Transition::Exit) => {
                                break;
                            }
                            Err(e) => return Err(e), // Task finished, state returned an Error
                        };
                    },

                    _ = i.tick() => {
                        println!("Interior tick");
                        continue;
                    }
                }
            }

            println!("[GameLoop::run] Game loop has finished.");

            Ok(())
        })
    }
}

// /// Example!
// #[tokio::main]
// async fn main() -> eyre::Result<()> {
//     let mut i = interval(Duration::from_millis(500));
//
//     let mut task = Platform::new(())
//         .system::<HeartbeatService>()
//         .system::<Heartbeat2Service>()
//         .ignite(lobby_state);
//
//     loop {
//         tokio::select! {
//             result = &mut task => {
//                 result?;
//                 break;
//             },
//             _ = i.tick() => {
//                 println!("Exterior loop! Showcasing we're cancellation safe!");
//             }
//         }
//     }
//
//     println!("Done!");
//
//     Ok(())
// }
//
