use insim::{builder::SpawnedHandle, core::track::Track, insim::RaceLaps};
use kitcar::{game, presence, scenes::{Scene, SceneError, SceneResult}};

/// Setup track
#[derive(Clone)]
pub struct SetupTrack {
    pub game: game::Game,
    pub presence: presence::Presence,
    pub insim: SpawnedHandle,
    pub min_players: usize,
    pub track: Track,
}

impl Scene for SetupTrack {
    type Output = ();

    async fn run(mut self) -> Result<SceneResult<()>, SceneError> {
        tokio::select! {
            _ = self.game.track_rotation(
                self.insim.clone(),
                self.track,
                RaceLaps::Practice,
                0,
                None,
            ) => {
                Ok(SceneResult::Continue(()))
            },
            _ = self.presence.wait_for_connection_count(|val| *val < self.min_players) => {
                tracing::info!("Lost players during track setup");
                Ok(SceneResult::bail_with("Lost players during SetupTrack"))
            }
        }
    }
}

