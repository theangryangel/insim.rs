use insim::{
    builder::InsimTask,
    core::{string::colours::Colour, track::Track},
    insim::RaceLaps,
};
use kitcar::{
    game, presence,
    scenes::{Scene, SceneError, SceneResult},
    ui,
};

use crate::components::topbar;

struct SetupTrackView {}
impl ui::Component for SetupTrackView {
    type Props = ();
    type Message = ();

    fn render(&self, _props: Self::Props) -> ui::Node<Self::Message> {
        topbar(&"Waiting for player ready".white())
    }
}

impl ui::View for SetupTrackView {
    type GlobalState = ();
    type ConnectionState = ();

    fn mount(_tx: tokio::sync::mpsc::UnboundedSender<Self::Message>) -> Self {
        Self {}
    }

    fn compose(_global: Self::GlobalState, _connection: Self::ConnectionState) -> Self::Props {
        ()
    }
}

/// Setup track
#[derive(Clone)]
pub struct SetupTrack {
    pub game: game::Game,
    pub presence: presence::Presence,
    pub insim: InsimTask,
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
