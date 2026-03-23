use insim::{builder::InsimTask, core::track::Track, insim::RaceLaps};
use kitcar::{
    game, presence,
    scenes::{FromContext, Scene, SceneError, SceneResult},
    ui,
};

use crate::hud::{topbar, Marquee, MarqueeProps};

struct SetupTrackView {
    marquee: Marquee,
    mode_name: String,
}
impl ui::Component for SetupTrackView {
    type Props<'a> = ();
    type Message = ();

    fn render(&self, _props: Self::Props<'_>) -> ui::Node<Self::Message> {
        ui::container()
            .flex()
            .flex_col()
            .w(200.)
            .with_child(
                topbar("Waiting for player ready").with_child(
                    self.marquee.render(MarqueeProps {
                        text: &self.mode_name,
                        width: 15,
                    })
                )
            )
    }
}

/// Setup track
#[derive(Clone)]
pub struct SetupTrack {
    pub min_players: usize,
    pub track: Track,
    pub layout: Option<String>,
    pub mode_name: String,
}

impl<Ctx> Scene<Ctx> for SetupTrack
where
    InsimTask: FromContext<Ctx>,
    game::Game: FromContext<Ctx>,
    presence::Presence: FromContext<Ctx>,
    Ctx: Sync,
{
    type Output = ();

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<()>, SceneError> {
        let insim = InsimTask::from_context(ctx);
        let mut game = game::Game::from_context(ctx);
        let mut presence = presence::Presence::from_context(ctx);

        let mode_name = self.mode_name.clone();
        let (_ui, _ui_handle) = ui::mount(insim.clone(), (), move |_ucid, invalidator| {
            let name = mode_name.clone();
            SetupTrackView {
                marquee: Marquee::new(invalidator),
                mode_name: name,
            }
        });
        tokio::select! {
            res = game.track_rotation(
                insim.clone(),
                self.track,
                RaceLaps::Practice,
                0,
                self.layout.clone(),
            ) => {
                res.map_err(|cause| SceneError::Custom {
                    scene: "setup_track::track_rotation",
                    cause: Box::new(cause),
                })?;
                Ok(SceneResult::Continue(()))
            },
            res = presence.wait_for_connection_count(|val| *val < self.min_players) => {
                let _ = res.map_err(|cause| SceneError::Custom {
                    scene: "setup_track::wait_for_connection_count",
                    cause: Box::new(cause),
                })?;
                tracing::info!("Lost players during track setup");
                Ok(SceneResult::bail_with("Lost players during SetupTrack"))
            }
        }
    }
}
