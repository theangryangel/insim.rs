//! MetronomeGame — MiniGame implementation for the metronome/event mode.

use std::time::Duration;

use kitcar::scenes::{Scene, SceneExt, SceneResult, SceneError, wait_for_players::WaitForPlayers};
use sqlx::types::Json;
use tokio::task::JoinHandle;

use super::{GameCtx, MiniGame};
use crate::{ChatError, MIN_PLAYERS, db, game_modes::metronome};
use super::setup_track;

#[derive(Clone)]
pub struct MetronomeGame {
    pub session_id: i64,
    pub start_round: usize,
    pub rounds: usize,
    pub target: Duration,
    pub max_scorers: usize,
    pub lobby_duration: Duration,
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

        let (rounds, target_ms, max_scorers, current_round, lobby_duration_secs) = match session.mode {
            Json(db::SessionMode::Metronome { rounds, target_ms, max_scorers, current_round, lobby_duration_secs }) => {
                (rounds, target_ms, max_scorers, current_round, lobby_duration_secs)
            },
            _ => unimplemented!(),
        };

        let start_round = (current_round as usize) + 1;

        let game = MetronomeGame {
            session_id: session.id,
            start_round,
            rounds: rounds as usize,
            target: Duration::from_millis(target_ms as u64),
            max_scorers: max_scorers as usize,
            lobby_duration: Duration::from_secs(lobby_duration_secs as u64),
            track: session.track,
            layout: session.layout.clone(),
            chat,
        };

        let guard = MetronomeGuard { chat_handle };
        Ok((game, guard))
    }

    async fn run(self, ctx: &GameCtx) -> Result<SceneResult<()>, SceneError> {
        // Setup: retry if players leave before the track loads
        loop {
            let setup = WaitForPlayers { min_players: MIN_PLAYERS }.then(
                setup_track::SetupTrack {
                    min_players: MIN_PLAYERS,
                    track: self.track,
                    layout: Some(self.layout.clone()),
                }
                .with_timeout(Duration::from_secs(60)),
            );

            match setup.run(ctx).await? {
                SceneResult::Continue(_) => break,
                SceneResult::Bail { .. } => continue,
                SceneResult::Quit => return Ok(SceneResult::Quit),
            }
        }

        let _spawn_control = crate::runner::spawn_control::spawn(ctx.insim.clone())
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "metronome::spawn_control",
                cause: Box::new(cause),
            })?;

        let event = metronome::Lobby {
            chat: self.chat.clone(),
            duration: self.lobby_duration,
        }
        .then(metronome::Rounds {
            chat: self.chat.clone(),
            start_round: self.start_round,
            rounds: self.rounds,
            target: self.target,
            max_scorers: self.max_scorers,
            session_id: self.session_id,
        })
        .then(metronome::Victory {
            session_id: self.session_id,
        });

        let presence = ctx.presence.clone();
        let chat = self.chat.clone();

        tokio::select! {
            res = event.run(ctx) => {
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
