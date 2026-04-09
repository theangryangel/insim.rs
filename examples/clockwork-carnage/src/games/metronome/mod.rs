//! MetronomeGame - open-format precision challenge.

mod challenge_loop;
pub mod chat;

use std::time::Duration;

pub use challenge_loop::ChallengeLoop;
use insim_extras::scenes::{
    IntoSceneError as _, Scene, SceneError, SceneExt, SceneResult, wait_for_players::WaitForPlayers,
};
use sqlx::types::Json;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::{MiniGame, MiniGameCtx, ordinal, position_xp, setup_track};
use crate::{ChatError, MIN_PLAYERS, db};

pub struct MetronomeGame {
    pub session_id: i64,
    pub target: Duration,
    pub track: insim::core::track::Track,
    pub layout: String,
    pub chat: chat::EventChat,
    pub event_name: Option<String>,
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

    async fn run(&self, ctx: &MiniGameCtx, cancel: CancellationToken) -> Result<(), SceneError> {
        let challenge_scene = WaitForPlayers {
            min_players: MIN_PLAYERS,
        }
        .then(|ctx: &MiniGameCtx| {
            let insim = ctx.insim.clone();
            async move {
                let _ = insim.send_message("Get ready!", None).await;
                Ok(SceneResult::Continue(()))
            }
        })
        .then(
            setup_track::SetupTrack {
                min_players: MIN_PLAYERS,
                track: self.track,
                layout: Some(self.layout.clone()),
                mode_name: match &self.event_name {
                    Some(name) => format!(
                        "{name} - Metronome: match the target lap time as closely as possible!"
                    ),
                    None => {
                        "Metronome: match the target lap time as closely as possible!".to_string()
                    },
                },
            }
            .with_timeout(Duration::from_secs(60)),
        )
        .then(
            ChallengeLoop {
                chat: self.chat.clone(),
                target: self.target,
                session_id: self.session_id,
                base_url: ctx.base_url.clone(),
            }
            .until_game_ends(),
        )
        .loop_until_quit()
        .with_cancellation(cancel);

        let _ = challenge_scene.run(ctx).await?;
        Ok(())
    }

    async fn setup(
        event: &db::Event,
        ctx: &MiniGameCtx,
    ) -> Result<(Self, Self::Guard), SceneError> {
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
            event_name: event.name.clone(),
        };

        let guard = MetronomeGuard { chat_handle };
        Ok((game, guard))
    }

    async fn teardown(&self, event: &db::Event, ctx: &MiniGameCtx) -> Result<(), SceneError> {
        db::complete_event(&ctx.pool, event.id)
            .await
            .scene_err("metronome::teardown")?;

        let standings = db::metronome_standings(&ctx.pool, event.id)
            .await
            .scene_err("metronome::teardown::standings")?;

        if !standings.is_empty() {
            let parts: Vec<String> = standings
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let xp = position_xp(i);
                    format!("{} {} (+{xp} XP)", ordinal(i + 1), s.pname)
                })
                .collect();
            let _ = ctx
                .insim
                .send_message(format!("Metronome: {}", parts.join(", ")), None)
                .await;

            for (i, s) in standings.iter().enumerate() {
                let xp = position_xp(i);
                if let Err(e) =
                    db::award_xp(&ctx.pool, &s.uname, xp, "metronome", Some(event.id)).await
                {
                    tracing::warn!("Failed to award XP to {}: {e}", s.uname);
                }
            }
        }

        Ok(())
    }
}
