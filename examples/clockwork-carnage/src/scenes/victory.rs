use std::time::Duration;

use insim::{builder::SpawnedHandle, core::string::colours::Colourify, insim::BtnStyle};
use kitcar::{
    presence,
    scenes::{Scene, SceneError, SceneResult},
    ui,
};
use tokio::time::sleep;

use crate::{
    components::{EnrichedLeaderboard, scoreboard, topbar},
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
impl ui::View for ClockworkVictoryView {
    type GlobalProps = ClockworkVictoryGlobalProps;
    type ConnectionProps = ClockworkVictoryConnectionProps;
    type Message = ();

    fn mount(_tx: tokio::sync::mpsc::UnboundedSender<Self::Message>) -> Self {
        Self {}
    }

    fn render(
        &self,
        global_props: Self::GlobalProps,
        connection_props: Self::ConnectionProps,
    ) -> ui::Node<Self::Message> {
        let players = scoreboard(&global_props.standings, &connection_props.uname);

        ui::container()
            .flex()
            .flex_col()
            .with_child(topbar("Final Standings! Thanks for playing!"))
            .with_child(
                ui::container()
                    .flex()
                    .mt(20.)
                    .w(200.)
                    .flex_col()
                    .items_start()
                    .with_child(
                        ui::text("Victory!".yellow(), BtnStyle::default().dark())
                            .w(35.)
                            .h(5.),
                    )
                    .with_children(players),
            )
    }
}

/// Victory scene - displays final standings
#[derive(Clone)]
pub struct Victory {
    pub insim: SpawnedHandle,
    pub presence: presence::Presence,
    pub scores: leaderboard::Leaderboard,
}

impl Scene for Victory {
    type Output = ();

    async fn run(self) -> Result<SceneResult<Self::Output>, SceneError> {
        let enriched_leaderboard = self.enriched_leaderboard().await;
        tracing::info!("leaderboard: {:?}", enriched_leaderboard);
        let ui = ui::attach::<ClockworkVictoryView>(
            self.insim.clone(),
            self.presence.clone(),
            ClockworkVictoryGlobalProps {
                standings: enriched_leaderboard.clone(),
            },
        );
        sleep(Duration::from_secs(120)).await;
        drop(ui);
        Ok(SceneResult::Continue(()))
    }
}

impl Victory {
    async fn enriched_leaderboard(&self) -> EnrichedLeaderboard {
        let ranking = self.scores.ranking();
        let names = self
            .presence
            .last_known_names(ranking.iter().map(|(uname, _)| uname))
            .await
            .unwrap_or_default();
        self.scores
            .ranking()
            .iter()
            .map(|(uname, pts)| {
                let pname = names.get(uname).cloned().unwrap_or_else(|| uname.clone());
                (uname.clone(), pname, *pts)
            })
            .collect()
    }
}
