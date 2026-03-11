//! Games: unified game executor with reconciliation loop.

pub mod bomb;
pub mod climb;
pub mod metronome;
pub mod setup_track;
pub mod shortcut;
pub mod spawn_control;

use insim::builder::InsimTask;
use kitcar::{game, presence, scenes::{FromContext, SceneError}};

use crate::db;

/// Shared context for all mini-games (persistent across mode switches).
pub struct GameCtx {
    pub pool: db::Pool,
    pub insim: InsimTask,
    pub presence: presence::Presence,
    pub game: game::Game,
}

impl FromContext<GameCtx> for InsimTask {
    fn from_context(ctx: &GameCtx) -> Self {
        ctx.insim.clone()
    }
}

impl FromContext<GameCtx> for presence::Presence {
    fn from_context(ctx: &GameCtx) -> Self {
        ctx.presence.clone()
    }
}

impl FromContext<GameCtx> for game::Game {
    fn from_context(ctx: &GameCtx) -> Self {
        ctx.game.clone()
    }
}

impl FromContext<GameCtx> for db::Pool {
    fn from_context(ctx: &GameCtx) -> Self {
        ctx.pool.clone()
    }
}

/// Lifecycle trait for mini-games. Each mode implements this.
pub trait MiniGame: Clone + Send + 'static {
    /// Resources to keep alive during the game (chat handles, etc.).
    /// Dropped after teardown to clean up background tasks.
    type Guard: Send + 'static;

    /// Initialize from an event. Creates/resumes DB entries, spawns
    /// mode-specific background tasks (e.g. chat handler).
    fn setup(
        event: &db::Event,
        ctx: &GameCtx,
    ) -> impl std::future::Future<Output = Result<(Self, Self::Guard), SceneError>> + Send;

    /// Run one iteration. Composes and executes the scene chain.
    fn run(
        self,
        ctx: &GameCtx,
    ) -> impl std::future::Future<Output = Result<kitcar::scenes::SceneResult<()>, SceneError>> + Send;

    /// Clean up: mark DB entries as ended.
    fn teardown(
        self,
        event: &db::Event,
        ctx: &GameCtx,
    ) -> impl std::future::Future<Output = Result<(), SceneError>> + Send;
}

/// Generic executor: setup, bail-retry loop, teardown.
pub async fn execute<G: MiniGame>(
    event: &db::Event,
    ctx: &GameCtx,
) -> Result<(), SceneError> {
    let (game, _guard) = G::setup(event, ctx).await?;
    loop {
        match game.clone().run(ctx).await? {
            kitcar::scenes::SceneResult::Continue(_) | kitcar::scenes::SceneResult::Quit => break,
            kitcar::scenes::SceneResult::Bail { msg } => {
                tracing::info!("Scene bailed ({msg:?}), retrying...");
                continue;
            },
        }
    }
    game.teardown(event, ctx).await?;
    ctx.insim.send_command("/axclear").await?;
    Ok(())
    // _guard dropped here -> chat JoinHandle aborted
}
