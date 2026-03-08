use std::time::Duration;

use insim::builder::InsimTask;
use kitcar::{
    scenes::{FromContext, Scene, SceneError, SceneResult},
    ui,
};
use tokio::time::sleep;

use crate::{
    hud::{EventLeaderboard, scoreboard, theme::hud_title, topbar},
    db,
};

#[derive(Debug, Clone, Default)]
struct ClockworkVictoryGlobalProps {
    standings: EventLeaderboard,
}

#[derive(Debug, Clone, Default)]
struct ClockworkVictoryConnectionProps {
    uname: String,
}

struct ClockworkVictoryView {}

#[derive(Debug, Clone, Default)]
struct ClockworkVictoryProps {
    global: ClockworkVictoryGlobalProps,
    connection: ClockworkVictoryConnectionProps,
}

impl ui::Component for ClockworkVictoryView {
    type Props = ClockworkVictoryProps;
    type Message = ();

    fn render(&self, props: Self::Props) -> ui::Node<Self::Message> {
        let players = scoreboard(&props.global.standings, &props.connection.uname);

        ui::container()
            .flex()
            .flex_col()
            .with_child(topbar("Final Standings! Thanks for playing!"))
            .with_child(
                ui::container()
                    .flex()
                    .mt(90.)
                    .pr(5.)
                    .w(200.)
                    .flex_col()
                    .items_end()
                    .with_child(ui::text("Victory!", hud_title()).w(35.).h(5.))
                    .with_children(players),
            )
    }
}

impl From<ui::UiState<ClockworkVictoryGlobalProps, ClockworkVictoryConnectionProps>>
    for ClockworkVictoryProps
{
    fn from(
        state: ui::UiState<ClockworkVictoryGlobalProps, ClockworkVictoryConnectionProps>,
    ) -> Self {
        Self {
            global: state.global,
            connection: state.connection,
        }
    }
}

/// Victory scene - displays final standings
#[derive(Clone)]
pub struct Victory {
    pub session_id: i64,
}

impl<Ctx> Scene<Ctx> for Victory
where
    InsimTask: FromContext<Ctx>,
    db::Pool: FromContext<Ctx>,
    Ctx: Sync,
{
    type Output = ();

    async fn run(self, ctx: &Ctx) -> Result<SceneResult<Self::Output>, SceneError> {
        let insim = InsimTask::from_context(ctx);
        let db = db::Pool::from_context(ctx);

        let leaderboard = event_leaderboard(&db, self.session_id).await?;
        tracing::info!("leaderboard: {:?}", leaderboard);
        let ui = ui::mount(
            insim,
            ClockworkVictoryGlobalProps {
                standings: leaderboard,
            },
            |_ucid, _invalidator| ClockworkVictoryView {},
        );
        sleep(Duration::from_secs(120)).await;
        drop(ui);
        Ok(SceneResult::Continue(()))
    }
}

async fn event_leaderboard(db: &db::Pool, session_id: i64) -> Result<EventLeaderboard, SceneError> {
    let standings = db::metronome_standings(db, session_id)
        .await
        .map_err(|cause| SceneError::Custom {
            scene: "victory::event_leaderboard",
            cause: Box::new(cause),
        })?;

    Ok(standings
        .into_iter()
        .map(|s| (s.uname, s.pname, s.total_points as u32))
        .collect::<Vec<_>>()
        .into())
}
