use std::time::Duration;

use insim::builder::InsimTask;
use kitcar::{
    game, presence,
    scenes::{FromContext, Scene, SceneError, SceneExt, SceneResult},
};

use super::chat;
use crate::db;

/// Clockwork Carnage game
#[derive(Clone)]
pub struct Clockwork {
    pub chat: chat::EventChat,

    pub start_round: usize,
    pub rounds: usize,
    pub target: Duration,
    pub max_scorers: usize,

    pub session_id: i64,
}

impl<Ctx> Scene<Ctx> for Clockwork
where
    InsimTask: FromContext<Ctx>,
    game::Game: FromContext<Ctx>,
    presence::Presence: FromContext<Ctx>,
    db::Pool: FromContext<Ctx>,
    Ctx: Sync,
{
    type Output = ();

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<()>, SceneError> {
        let insim = InsimTask::from_context(ctx);
        let mut game = game::Game::from_context(ctx);
        let presence = presence::Presence::from_context(ctx);

        let _spawn_control = crate::runner::spawn_control::spawn(insim.clone())
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "clockwork::spawn_control",
                cause: Box::new(cause),
            })?;

        // Scenes inside scenes inside scenes..
        let event = super::Lobby {
            chat: self.chat.clone(),
        }
        .then(super::Rounds {
            chat: self.chat.clone(),
            start_round: self.start_round,
            rounds: self.rounds,
            target: self.target,
            max_scorers: self.max_scorers,
            session_id: self.session_id,
        })
        .then(super::Victory {
            session_id: self.session_id,
        });

        tokio::select! {
            res = event.run(ctx) => {
                let _ = res?;
                Ok(SceneResult::Continue(()))
            },
            _ = self.chat.wait_for_admin_cmd(presence.clone(), |msg| matches!(msg, chat::EventChatMsg::End)) => {
                tracing::info!("Admin ended event");
                Ok(SceneResult::bail_with("Admin ended event"))
            },
            _ = game.wait_for_end() => {
                tracing::info!("Players voted to end");
                Ok(SceneResult::Continue(()))
            }
        }
    }
}
