//! Games: unified game executor with reconciliation loop.

pub mod bomb;
pub mod manager;
pub mod metronome;
pub mod setup_track;
pub mod shortcut;
// pub mod spawn_control;
pub mod vehicle_restrictions;

use insim::builder::InsimTask;
use kitcar::{
    game, presence,
    scenes::{FromContext, SceneError},
};
pub use manager::MiniGameManager;
use tokio_util::sync::CancellationToken;

use crate::db;

/// Shared context for all mini-games (persistent across mode switches).
pub struct MiniGameCtx {
    pub pool: db::Pool,
    pub insim: InsimTask,
    pub presence: presence::Presence,
    pub game: game::Game,
    pub base_url: Option<String>,
}

impl FromContext<MiniGameCtx> for InsimTask {
    fn from_context(ctx: &MiniGameCtx) -> Self {
        ctx.insim.clone()
    }
}

impl FromContext<MiniGameCtx> for presence::Presence {
    fn from_context(ctx: &MiniGameCtx) -> Self {
        ctx.presence.clone()
    }
}

impl FromContext<MiniGameCtx> for game::Game {
    fn from_context(ctx: &MiniGameCtx) -> Self {
        ctx.game.clone()
    }
}

impl FromContext<MiniGameCtx> for db::Pool {
    fn from_context(ctx: &MiniGameCtx) -> Self {
        ctx.pool.clone()
    }
}

/// Lifecycle trait for mini-games. Each mode implements this.
///
/// `run` owns the cancellation token and applies it internally so that `execute` stays clean.
/// `async_fn_in_trait`: Send is verified at concrete impl sites — `execute<G>` is a Send async fn.
#[allow(async_fn_in_trait)]
pub trait MiniGame: Send + Sized + 'static {
    /// Resources to keep alive during the game (chat handles, etc.).
    /// Dropped after teardown to clean up background tasks.
    type Guard: Send + 'static;

    /// Initialize from an event. Creates/resumes DB entries, spawns
    /// mode-specific background tasks (e.g. chat handler).
    fn setup(
        event: &db::Event,
        ctx: &MiniGameCtx,
    ) -> impl std::future::Future<Output = Result<(Self, Self::Guard), SceneError>> + Send;

    /// Run the game scene chain to completion or until `cancel` fires.
    async fn run(&self, ctx: &MiniGameCtx, cancel: CancellationToken) -> Result<(), SceneError>;

    /// Clean up: mark DB entries as ended.
    async fn teardown(&self, event: &db::Event, ctx: &MiniGameCtx) -> Result<(), SceneError>;
}
