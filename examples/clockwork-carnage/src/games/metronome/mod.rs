//! MetronomeGame — open-format precision challenge.

mod challenge_loop;
pub mod chat;

use std::time::Duration;

pub use challenge_loop::ChallengeLoop;
use kitcar::scenes::{Scene, SceneError, SceneExt, SceneResult, wait_for_players::WaitForPlayers};
use sqlx::types::Json;
use tokio::task::JoinHandle;

use super::{GameCtx, MiniGame, setup_track};
use crate::{ChatError, MIN_PLAYERS, db};

#[derive(Clone)]
pub struct MetronomeGame {
    pub session_id: i64,
    pub target: Duration,
    pub track: insim::core::track::Track,
    pub layout: String,
    pub chat: chat::EventChat,
}

pub struct MetronomeGuard {
    chat_handle: JoinHandle<Result<(), ChatError>>,
}

impl Drop for MetronomeGuard {
    fn drop(&mut self) {
        self.chat_handle.abort();
    }
}

impl MiniGame for MetronomeGame {
    type Guard = MetronomeGuard;

    async fn setup(event: &db::Event, ctx: &GameCtx) -> Result<(Self, Self::Guard), SceneError> {
        let (chat, chat_handle) = chat::spawn(ctx.insim.clone());

        let target_ms = match event.mode {
            Json(db::EventMode::Metronome { target_ms }) => target_ms,
            _ => unimplemented!(),
        };

        let game = MetronomeGame {
            session_id: event.id,
            target: Duration::from_millis(target_ms as u64),
            track: event.track,
            layout: event.layout.clone(),
            chat,
        };

        let guard = MetronomeGuard { chat_handle };
        Ok((game, guard))
    }

    async fn run(self, ctx: &GameCtx) -> Result<SceneResult<()>, SceneError> {
        let challenge_scene = WaitForPlayers {
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
        .then(ChallengeLoop {
            chat: self.chat.clone(),
            target: self.target,
            session_id: self.session_id,
        })
        .loop_until_quit();

        let presence = ctx.presence.clone();
        let chat = self.chat.clone();

        tokio::select! {
            res = challenge_scene.run(ctx) => {
                let _ = res?;
                Ok(SceneResult::Continue(()))
            },
            _ = chat.wait_for_admin_cmd(presence, |msg| matches!(msg, chat::EventChatMsg::Quit)) => {
                Ok(SceneResult::Quit)
            }
        }
    }

    async fn teardown(self, event: &db::Event, ctx: &GameCtx) -> Result<(), SceneError> {
        db::complete_event(&ctx.pool, event.id)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "metronome::teardown",
                cause: Box::new(cause),
            })
    }
}
