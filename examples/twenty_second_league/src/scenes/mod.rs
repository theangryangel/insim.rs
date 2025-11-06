//! Phases
mod game;
mod idle;
mod lobby;
mod track_rotation;

pub(crate) use game::{Round, Victory};
pub(crate) use idle::Idle;
pub(crate) use lobby::Lobby;
use tokio::task::JoinHandle;
pub(crate) use track_rotation::TrackRotation;

#[derive(Debug, Clone, from_variants::FromVariants)]
pub enum Scene {
    Idle(Idle),
    TrackRotation(TrackRotation),
    Lobby(Lobby),
    Round(Round),
    Victory(Victory),
}

impl Scene {
    pub fn spawn(self, cx: super::Context) -> JoinHandle<anyhow::Result<Option<Scene>>> {
        tokio::task::spawn(async move {
            match self {
                Scene::Idle(idle) => idle.run(cx).await,
                Scene::TrackRotation(track_rotation) => track_rotation.run(cx).await,
                Scene::Lobby(lobby) => lobby.run(cx).await,
                Scene::Round(round) => round.run(cx).await,
                Scene::Victory(victory) => victory.run(cx).await,
            }
        })
    }
}
