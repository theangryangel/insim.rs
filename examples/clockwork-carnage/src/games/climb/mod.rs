//! ClimbGame — MiniGame implementation for the climb mode.

mod challenge_loop;
pub mod chat;

pub use challenge_loop::ClimbLoop;

use std::time::Duration;

use kitcar::scenes::{Scene, SceneExt, SceneError, SceneResult, wait_for_players::WaitForPlayers};
use tokio::task::JoinHandle;

use super::{GameCtx, MiniGame};
use crate::{ChatError, MIN_PLAYERS, db};
use super::setup_track;

#[derive(Clone)]
pub struct ClimbGame {
    pub session_id: i64,
    pub track: insim::core::track::Track,
    pub layout: String,
    pub chat: chat::ClimbChat,
}

pub struct ClimbGuard {
    chat_handle: JoinHandle<Result<(), ChatError>>,
}

impl Drop for ClimbGuard {
    fn drop(&mut self) {
        self.chat_handle.abort();
    }
}

impl MiniGame for ClimbGame {
    type Guard = ClimbGuard;

    async fn setup(event: &db::Event, ctx: &GameCtx) -> Result<(Self, Self::Guard), SceneError> {
        let (chat, chat_handle) = chat::spawn(ctx.insim.clone());

        let game = ClimbGame {
            session_id: event.id,
            track: event.track,
            layout: event.layout.clone(),
            chat,
        };

        let guard = ClimbGuard { chat_handle };
        Ok((game, guard))
    }

    async fn run(self, ctx: &GameCtx) -> Result<SceneResult<()>, SceneError> {
        let climb_scene = WaitForPlayers {
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
        .then(ClimbLoop {
            chat: self.chat.clone(),
            session_id: self.session_id,
        })
        .loop_until_quit();

        let presence = ctx.presence.clone();
        let chat = self.chat.clone();

        tokio::select! {
            res = climb_scene.run(ctx) => {
                let _ = res?;
                Ok(SceneResult::Continue(()))
            },
            _ = chat.wait_for_admin_cmd(presence, |msg| matches!(msg, chat::ClimbChatMsg::Quit)) => {
                Ok(SceneResult::Quit)
            }
        }
    }

    async fn teardown(self, event: &db::Event, ctx: &GameCtx) -> Result<(), SceneError> {
        db::complete_event(&ctx.pool, event.id)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "climb::teardown",
                cause: Box::new(cause),
            })
    }
}
