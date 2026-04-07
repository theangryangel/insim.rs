//! BombGame - MiniGame implementation for bomb/countdown mode.

mod challenge_loop;
pub mod chat;
pub mod state;

use std::time::Duration;

pub use challenge_loop::BombLoop;
use insim_extras::scenes::{Scene, SceneError, SceneExt, wait_for_players::WaitForPlayers};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::{MiniGame, MiniGameCtx, setup_track};
use crate::{ChatError, MIN_PLAYERS, db};

pub struct BombGame {
    pub session_id: i64,
    pub track: insim::core::track::Track,
    pub layout: String,
    pub checkpoint_timeout: Duration,
    pub checkpoint_penalty: Duration,
    pub collision_max_penalty: Duration,
    pub chat: chat::BombChat,
    pub event_name: Option<String>,
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

    async fn run(&self, ctx: &MiniGameCtx, cancel: CancellationToken) -> Result<(), SceneError> {
        let bomb_scene = WaitForPlayers {
            min_players: MIN_PLAYERS,
        }
        .then(
            setup_track::SetupTrack {
                min_players: MIN_PLAYERS,
                track: self.track,
                layout: Some(self.layout.clone()),
                mode_name: match &self.event_name {
                    Some(name) => {
                        format!("{name} - Bomb: hit every checkpoint before the clock runs out!")
                    },
                    None => "Bomb: hit every checkpoint before the clock runs out!".to_string(),
                },
            }
            .with_timeout(Duration::from_secs(60)),
        )
        .then(
            BombLoop {
                chat: self.chat.clone(),
                session_id: self.session_id,
                checkpoint_timeout: self.checkpoint_timeout,
                checkpoint_penalty: self.checkpoint_penalty,
                collision_max_penalty: self.collision_max_penalty,
                base_url: ctx.base_url.clone(),
            }
            .until_game_ends(),
        )
        .loop_until_quit()
        .with_cancellation(cancel);

        let _ = bomb_scene.run(ctx).await?;
        Ok(())
    }

    async fn setup(
        event: &db::Event,
        ctx: &MiniGameCtx,
    ) -> Result<(Self, Self::Guard), SceneError> {
        let (checkpoint_timeout, checkpoint_penalty, collision_max_penalty) = match *event.mode {
            db::EventMode::Bomb {
                checkpoint_timeout_secs,
                checkpoint_penalty_ms,
                collision_max_penalty_ms,
            } => (
                Duration::from_secs(checkpoint_timeout_secs as u64),
                Duration::from_millis(checkpoint_penalty_ms as u64),
                Duration::from_millis(collision_max_penalty_ms as u64),
            ),
            _ => (
                Duration::from_secs(30),
                Duration::from_millis(250),
                Duration::from_millis(500),
            ),
        };

        let (chat, chat_handle) = chat::spawn(ctx.insim.clone());

        let game = BombGame {
            session_id: event.id,
            track: event.track,
            layout: event.layout.clone(),
            checkpoint_timeout,
            checkpoint_penalty,
            collision_max_penalty,
            chat,
            event_name: event.name.clone(),
        };

        let guard = BombGuard { chat_handle };
        Ok((game, guard))
    }

    async fn teardown(&self, event: &db::Event, ctx: &MiniGameCtx) -> Result<(), SceneError> {
        db::complete_event(&ctx.pool, event.id)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "bomb::teardown",
                cause: Box::new(cause),
            })?;

        let standings = db::bomb_best_runs(&ctx.pool, event.id)
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "bomb::teardown::standings",
                cause: Box::new(cause),
            })?;

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
                .send_message(format!("Bomb: {}", parts.join(", ")), None)
                .await;

            for (i, s) in standings.iter().enumerate() {
                let xp = position_xp(i);
                if let Err(e) = db::award_xp(&ctx.pool, &s.uname, xp, "bomb", Some(event.id)).await
                {
                    tracing::warn!("Failed to award XP to {}: {e}", s.uname);
                }
            }
        }

        Ok(())
    }
}

fn position_xp(rank: usize) -> i64 {
    match rank {
        0 => 100,
        1 => 75,
        2 => 50,
        3..=9 => 25,
        _ => 10,
    }
}

fn ordinal(n: usize) -> String {
    let suffix = match n % 10 {
        1 if n % 100 != 11 => "st",
        2 if n % 100 != 12 => "nd",
        3 if n % 100 != 13 => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}
