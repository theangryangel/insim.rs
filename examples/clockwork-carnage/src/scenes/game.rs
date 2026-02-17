use std::time::Duration;

use insim::builder::InsimTask;
use kitcar::{
    game, presence,
    scenes::{Scene, SceneError, SceneExt, SceneResult},
};

use crate::chat;

/// Clockwork Carnage game
#[derive(Clone)]
pub struct Clockwork {
    pub game: game::Game,
    pub presence: presence::Presence,
    pub chat: chat::Chat,
    pub insim: InsimTask,

    pub rounds: usize,
    pub target: Duration,
    pub max_scorers: usize,
}

impl Scene for Clockwork {
    type Output = ();

    async fn run(mut self) -> Result<SceneResult<()>, SceneError> {
        let _spawn_control = crate::spawn_control::spawn(self.insim.clone())
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
            rounds: self.rounds,
            target: self.target,
            max_scorers: self.max_scorers,
        })
        .and_then({
            let insim = self.insim.clone();
            let presence = self.presence.clone();
            move |scores| {
                tracing::info!("scores = {:?}", scores);
                super::Victory {
                    insim: insim.clone(),
                    presence: presence.clone(),
                    scores,
                }
            }
        });

        tokio::select! {
            res = event.run() => {
                let _ = res?;
                Ok(SceneResult::Continue(()))
            },
            // TODO: if this all we care about.. do we want to handle this here? it's contextually
            // sensitive.. so maybe this is the right place?
            _ = self.chat.wait_for_admin_cmd(self.presence.clone(), |msg| matches!(msg, chat::ChatMsg::End)) => {
                tracing::info!("Admin ended event");
                Ok(SceneResult::bail_with("Admin ended event"))
            },
            // XXX: not required with `/vote no` in main *technically*, however an admin can still /end!
            _ = self.game.wait_for_end() => {
                tracing::info!("Players voted to end");
                Ok(SceneResult::Continue(()))
            }
        }
    }
}
