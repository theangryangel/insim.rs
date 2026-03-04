//! ShortcutGame — MiniGame implementation for the shortcut/challenge mode.

use std::time::Duration;

use kitcar::scenes::{Scene, SceneExt, SceneResult, SceneError, wait_for_players::WaitForPlayers};
use tokio::task::JoinHandle;

use super::{GameCtx, MiniGame};
use crate::{ChatError, MIN_PLAYERS, db, setup_track, shortcut};

#[derive(Clone)]
pub struct ShortcutGame {
    pub session_id: i64,
    pub track: insim::core::track::Track,
    pub layout: String,
    pub chat: shortcut::chat::ChallengeChat,
}

pub struct ShortcutGuard {
    chat_handle: JoinHandle<Result<(), ChatError>>,
}

impl Drop for ShortcutGuard {
    fn drop(&mut self) {
        self.chat_handle.abort();
    }
}

impl MiniGame for ShortcutGame {
    type Guard = ShortcutGuard;

    async fn setup(session: &db::Session, ctx: &GameCtx) -> Result<(Self, Self::Guard), SceneError> {
        let (chat, chat_handle) = shortcut::chat::spawn(ctx.insim.clone());

        let game = ShortcutGame {
            session_id: session.id,
            track: session.track,
            layout: session.layout.clone(),
            chat,
        };

        let guard = ShortcutGuard { chat_handle };
        Ok((game, guard))
    }

    async fn run(self, ctx: &GameCtx) -> Result<SceneResult<()>, SceneError> {
        let challenge_scene = WaitForPlayers {
            insim: ctx.insim.clone(),
            presence: ctx.presence.clone(),
            min_players: MIN_PLAYERS,
        }
        .then(
            setup_track::SetupTrack {
                insim: ctx.insim.clone(),
                presence: ctx.presence.clone(),
                min_players: MIN_PLAYERS,
                game: ctx.game.clone(),
                track: self.track,
                layout: Some(self.layout.clone()),
            }
            .with_timeout(Duration::from_secs(60)),
        )
        .then(shortcut::ChallengeLoop {
            insim: ctx.insim.clone(),
            game: ctx.game.clone(),
            presence: ctx.presence.clone(),
            chat: self.chat.clone(),
            db: ctx.pool.clone(),
            session_id: self.session_id,
        })
        .loop_until_quit();

        let presence = ctx.presence.clone();
        let chat = self.chat.clone();

        tokio::select! {
            res = challenge_scene.run() => {
                let _ = res?;
                Ok(SceneResult::Continue(()))
            },
            _ = chat.wait_for_admin_cmd(presence, |msg| matches!(msg, shortcut::chat::ChallengeChatMsg::Quit)) => {
                Ok(SceneResult::Quit)
            }
        }
    }

    async fn teardown(self, session: &db::Session, ctx: &GameCtx) -> Result<(), SceneError> {
        db::complete_session(&ctx.pool, session.id)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "shortcut::teardown",
                cause: Box::new(cause),
            })
    }
}
