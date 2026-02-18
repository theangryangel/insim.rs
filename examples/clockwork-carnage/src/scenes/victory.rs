use std::time::Duration;

use insim::builder::InsimTask;
use kitcar::{
    presence,
    scenes::{Scene, SceneError, SceneResult},
    ui,
};
use tokio::time::sleep;

use crate::{
    components::{EnrichedLeaderboard, scoreboard, theme::hud_title, topbar},
    leaderboard,
};

#[derive(Debug, Clone, Default)]
struct ClockworkVictoryGlobalProps {
    standings: EnrichedLeaderboard,
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
    pub insim: InsimTask,
    pub presence: presence::Presence,
    pub scores: leaderboard::Leaderboard,
}

impl Scene for Victory {
    type Output = ();

    async fn run(self) -> Result<SceneResult<Self::Output>, SceneError> {
        let enriched_leaderboard = self.enriched_leaderboard().await?;
        tracing::info!("leaderboard: {:?}", enriched_leaderboard);
        let ui = ui::mount(
            self.insim.clone(),
            ClockworkVictoryGlobalProps {
                standings: enriched_leaderboard.clone(),
            },
            |_ucid, _invalidator| ClockworkVictoryView {},
        );
        sleep(Duration::from_secs(120)).await;
        drop(ui);
        Ok(SceneResult::Continue(()))
    }
}

impl Victory {
    async fn enriched_leaderboard(&self) -> Result<EnrichedLeaderboard, SceneError> {
        let ranking = self.scores.ranking();
        let names = self
            .presence
            .last_known_names(ranking.iter().map(|(uname, _)| uname))
            .await
            .map_err(|cause| SceneError::Custom {
                scene: "victory::enriched_leaderboard::last_known_names",
                cause: Box::new(cause),
            })?;

        Ok(ranking
            .iter()
            .map(|(uname, pts)| {
                let pname = names.get(uname).cloned().unwrap_or_else(|| uname.clone());
                (uname.clone(), pname, *pts)
            })
            .collect::<Vec<_>>()
            .into())
    }
}
