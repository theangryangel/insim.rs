use std::time::Duration;

use insim::builder::InsimTask;
use kitcar::{
    game, presence,
    scenes::{Scene, SceneError, SceneExt, SceneResult},
};

use super::chat;
use crate::db;

/// Clockwork Carnage game
#[derive(Clone)]
pub struct Clockwork {
    pub game: game::Game,
    pub presence: presence::Presence,
    pub chat: chat::EventChat,
    pub insim: InsimTask,

    pub start_round: usize,
    pub rounds: usize,
    pub target: Duration,
    pub max_scorers: usize,

    pub db: db::Pool,
    pub session_id: i64,
}

impl Scene for Clockwork {
    type Output = ();

    async fn run(mut self) -> Result<SceneResult<()>, SceneError> {
        let _spawn_control = crate::runner::spawn_control::spawn(self.insim.clone())
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "clockwork::spawn_control",
                cause: Box::new(cause),
            })?;

        // Scenes inside scenes inside scenes..
        let event = super::Lobby {
            insim: self.insim.clone(),
            chat: self.chat.clone(),
        }
        .then(super::Rounds {
            insim: self.insim.clone(),
            game: self.game.clone(),
            presence: self.presence.clone(),
            chat: self.chat.clone(),
            start_round: self.start_round,
            rounds: self.rounds,
            target: self.target,
            max_scorers: self.max_scorers,
            db: self.db.clone(),
            session_id: self.session_id,
        })
        .then(super::Victory {
            insim: self.insim.clone(),
            db: self.db.clone(),
            session_id: self.session_id,
        });

        tokio::select! {
            res = event.run() => {
                let _ = res?;
                Ok(SceneResult::Continue(()))
            },
            _ = self.chat.wait_for_admin_cmd(self.presence.clone(), |msg| matches!(msg, chat::EventChatMsg::End)) => {
                tracing::info!("Admin ended event");
                Ok(SceneResult::bail_with("Admin ended event"))
            },
            _ = self.game.wait_for_end() => {
                tracing::info!("Players voted to end");
                Ok(SceneResult::Continue(()))
            }
        }
    }
}
