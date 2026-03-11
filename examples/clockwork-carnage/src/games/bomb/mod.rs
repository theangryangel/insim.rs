//! BombGame — MiniGame implementation for bomb/countdown mode.

mod challenge_loop;
pub mod chat;

pub use challenge_loop::BombLoop;

use std::time::Duration;

use kitcar::scenes::{Scene, SceneExt, SceneResult, SceneError, wait_for_players::WaitForPlayers};
use tokio::task::JoinHandle;

use super::{GameCtx, MiniGame};
use crate::{ChatError, MIN_PLAYERS, db};
use super::setup_track;

#[derive(Clone)]
pub struct BombGame {
    pub session_id: i64,
    pub track: insim::core::track::Track,
    pub layout: String,
    pub checkpoint_timeout: Duration,
    pub chat: chat::BombChat,
}

pub struct BombGuard {
    chat_handle: JoinHandle<Result<(), ChatError>>,
}

impl Drop for BombGuard {
    fn drop(&mut self) {
        self.chat_handle.abort();
    }
}

impl MiniGame for BombGame {
    type Guard = BombGuard;

    async fn setup(event: &db::Event, ctx: &GameCtx) -> Result<(Self, Self::Guard), SceneError> {
        let checkpoint_timeout = match *event.mode {
            db::EventMode::Bomb { checkpoint_timeout_secs } => {
                Duration::from_secs(checkpoint_timeout_secs as u64)
            },
            _ => Duration::from_secs(30),
        };

        let (chat, chat_handle) = chat::spawn(ctx.insim.clone());

        let game = BombGame {
            session_id: event.id,
            track: event.track,
            layout: event.layout.clone(),
            checkpoint_timeout,
            chat,
        };

        let guard = BombGuard { chat_handle };
        Ok((game, guard))
    }

    async fn run(self, ctx: &GameCtx) -> Result<SceneResult<()>, SceneError> {
        let bomb_scene = WaitForPlayers {
            min_players: MIN_PLAYERS,
        }
        .then(
            setup_track::SetupTrack {
                min_players: MIN_PLAYERS,
                track: self.track,
                layout: Some(self.layout.clone()),
            }
            .with_timeout(Duration::from_secs(60)),
        )
        .then(BombLoop {
            chat: self.chat.clone(),
            session_id: self.session_id,
            checkpoint_timeout: self.checkpoint_timeout,
        })
        .loop_until_quit();

        let presence = ctx.presence.clone();
        let chat = self.chat.clone();

        tokio::select! {
            res = bomb_scene.run(ctx) => {
                let _ = res?;
                Ok(SceneResult::Continue(()))
            },
            _ = chat.wait_for_admin_cmd(presence, |msg| matches!(msg, chat::BombChatMsg::Quit)) => {
                Ok(SceneResult::Quit)
            }
        }
    }

    async fn teardown(self, event: &db::Event, ctx: &GameCtx) -> Result<(), SceneError> {
        db::complete_event(&ctx.pool, event.id)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "bomb::teardown",
                cause: Box::new(cause),
            })
    }
}
