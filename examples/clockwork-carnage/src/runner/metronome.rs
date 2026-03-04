//! MetronomeGame — MiniGame implementation for the metronome/event mode.

use std::time::Duration;

use kitcar::scenes::{Scene, SceneExt, SceneResult, SceneError, wait_for_players::WaitForPlayers};
use sqlx::types::Json;
use tokio::task::JoinHandle;

use super::{GameCtx, MiniGame};
use crate::{ChatError, MIN_PLAYERS, db, metronome, setup_track};

#[derive(Clone)]
pub struct MetronomeGame {
    pub session_id: i64,
    pub start_round: usize,
    pub rounds: usize,
    pub target: Duration,
    pub max_scorers: usize,
    pub track: insim::core::track::Track,
    pub layout: String,
    pub chat: metronome::chat::EventChat,
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

    async fn setup(session: &db::Session, ctx: &GameCtx) -> Result<(Self, Self::Guard), SceneError> {
        let (chat, chat_handle) = metronome::chat::spawn(ctx.insim.clone());

        // Check if metronome_sessions row exists (crash recovery)
        let session = db::get_session(&ctx.pool, session.id)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "metronome::setup::get_metronome_session",
                cause: Box::new(cause),
            })?
            .ok_or_else(|| SceneError::Custom {
                scene: "metronome::setup",
                cause: "no metronome_sessions row for this session".into(),
            })?;

        let (rounds, target_ms, max_scorers, current_round) = match session.mode {
            Json(db::SessionMode::Metronome { rounds, target_ms, max_scorers, current_round }) => (rounds, target_ms, max_scorers, current_round),
            _ => {
                unimplemented!()
            }
        };

        let start_round = (current_round as usize) + 1;

        let game = MetronomeGame {
            session_id: session.id,
            start_round,
            rounds: rounds as usize,
            target: Duration::from_millis(target_ms as u64),
            max_scorers: max_scorers as usize,
            track: session.track,
            layout: session.layout.clone(),
            chat,
        };

        let guard = MetronomeGuard { chat_handle };
        Ok((game, guard))
    }

    async fn run(self, ctx: &GameCtx) -> Result<SceneResult<()>, SceneError> {
        let clockwork = WaitForPlayers {
            insim: ctx.insim.clone(),
            presence: ctx.presence.clone(),
            min_players: MIN_PLAYERS,
        }
        .then(metronome::WaitForAdminStart {
            insim: ctx.insim.clone(),
            presence: ctx.presence.clone(),
            chat: self.chat.clone(),
        })
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
        .then(metronome::Clockwork {
            game: ctx.game.clone(),
            presence: ctx.presence.clone(),
            chat: self.chat.clone(),
            start_round: self.start_round,
            rounds: self.rounds,
            max_scorers: self.max_scorers,
            target: self.target,
            insim: ctx.insim.clone(),
            db: ctx.pool.clone(),
            session_id: self.session_id,
        })
        .loop_until_quit();

        let presence = ctx.presence.clone();
        let chat = self.chat.clone();

        tokio::select! {
            res = clockwork.run() => {
                let _ = res?;
                Ok(SceneResult::Continue(()))
            },
            _ = chat.wait_for_admin_cmd(presence, |msg| matches!(msg, metronome::chat::EventChatMsg::Quit)) => {
                Ok(SceneResult::Quit)
            }
        }
    }

    async fn teardown(self, session: &db::Session, ctx: &GameCtx) -> Result<(), SceneError> {
        db::complete_session(&ctx.pool, session.id)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "metronome::teardown",
                cause: Box::new(cause),
            })
    }
}
