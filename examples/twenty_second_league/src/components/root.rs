use std::time::Duration;

use insim::core::string::colours::Colourify;
use kitcar::{
    combos::Combo,
    ui::{Element, Scope, component},
};

const WELCOME: &str = "Welcome drivers!
Forget being the fastest, the goal is to be the most precise. Finish in as close to 20secs as possible!
Full contact is allowed.
Just remember: Don't be a dick. We're all here to have fun!";

use crate::{
    combo::ComboExt,
    components::{
        leaderboard::{Leaderboard, LeaderboardProps},
        motd::{Motd, MotdProps},
        topbar::{Topbar, TopbarProps},
    },
    db::models::LeaderboardEntry,
};

#[derive(Debug, Clone)]
pub enum RootScene {
    Idle,
    TrackRotation {
        combo: Combo<ComboExt>,
    },
    Round {
        combo: Combo<ComboExt>,
        round: u32,
        remaining: Duration,
        scores: Vec<LeaderboardEntry>,
    },
    Lobby {
        combo: Combo<ComboExt>,
        remaining: Duration,
    },
    Victory {
        remaining: Duration,
    },
}

#[component]
pub(crate) fn Root(scene: RootScene) -> Option<Element> {
    let text = match &scene {
        RootScene::Idle => "No game in progress".white(),
        RootScene::TrackRotation { combo } => format!("Loading track {}", combo.track(),).white(),
        RootScene::Round {
            round,
            combo,
            remaining,
            ..
        } => {
            let seconds = remaining.as_secs() % 60;
            let minutes = (remaining.as_secs() / 60) % 60;
            format!(
                "Round {}/{} · {:02}:{:02} remaining",
                round,
                combo.extensions().rounds,
                minutes,
                seconds
            )
            .white()
        },
        RootScene::Lobby { remaining, combo } => {
            let seconds = remaining.as_secs() % 60;
            let minutes = (remaining.as_secs() / 60) % 60;
            format!(
                "Warmup · {:02}:{:02} before {} rounds",
                minutes,
                seconds,
                combo.extensions().rounds
            )
            .white()
        },
        RootScene::Victory { remaining } => {
            let seconds = remaining.as_secs() % 60;
            let minutes = (remaining.as_secs() / 60) % 60;
            format!("Thanks for playing · {:02}:{:02}", minutes, seconds).white()
        },
    };

    let leaderboard_entries = match &scene {
        RootScene::Round { scores, .. } => scores.clone(),
        _ => vec![],
    };

    let interface = cx
        .container()
        .h(150.0)
        .w(200.0)
        .flex()
        .flex_col()
        .with_child(cx.component::<Topbar>(TopbarProps { text }))
        .with_child(
            // FIXME: this is wrong.
            cx.container()
                .flex()
                .flex_row()
                .with_child(cx.component::<Motd>(MotdProps {
                    text: WELCOME.to_owned(),
                }))
                .with_child(
                    cx.container()
                        .flex()
                        .flex_col()
                        // FIXME: float right?
                        .mr(2.0)
                        .mt(2.0)
                        .with_child(cx.component::<Leaderboard>(LeaderboardProps {
                            entries: leaderboard_entries,
                        })),
                ),
        );

    Some(interface)
}
