use insim::{
    builder::SpawnedHandle,
    core::{string::colours::Colourify, track::Track},
    insim::RaceLaps,
};
use kitcar::{
    game, presence,
    scenes::{Scene, SceneError, SceneResult},
    ui,
};

use super::topbar;

struct SetupTrackView {}
impl ui::View for SetupTrackView {
    type GlobalProps = ();
    type ConnectionProps = ();
    type Message = ();

    fn mount(_tx: tokio::sync::mpsc::UnboundedSender<Self::Message>) -> Self {
        Self {}
    }

    fn render(
        &self,
        _global_props: Self::GlobalProps,
        _connection_props: Self::ConnectionProps,
    ) -> ui::Node<Self::Message> {
        topbar::topbar(&"Waiting for player ready".white())
    }
}

/// Setup track
#[derive(Clone)]
pub struct SetupTrack {
    pub game: game::Game,
    pub presence: presence::Presence,
    pub insim: SpawnedHandle,
    pub min_players: usize,
    pub track: Track,
    pub layout: Option<String>,
}

impl Scene for SetupTrack {
    type Output = ();

    async fn run(mut self) -> Result<SceneResult<()>, SceneError> {
        let _ = ui::attach::<SetupTrackView>(self.insim.clone(), self.presence.clone(), ());
        tokio::select! {
            _ = self.game.track_rotation(
                self.insim.clone(),
                self.track,
                RaceLaps::Practice,
                0,
                self.layout.clone(),
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
