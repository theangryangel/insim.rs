use std::time::Duration;

use insim::core::string::colours::Colourify;
use kitcar::ui::{Element, Scope, component};

const WELCOME: &str = "Welcome drivers!
Forget being the fastest, the goal is to be the most precise. Finish in as close to 20secs as possible!
Full contact is allowed.
Just remember: Don't be a dick. We're all here to have fun!";

use crate::{components::{
    motd::{Motd, MotdProps},
    topbar::{Topbar, TopbarProps},
}, GameState};

#[derive(Debug, Clone)]
pub enum RootPhase { // FIXME: remove this and replace with GameState being passed into the UI
    Idle,
    Restarting,
    Game {
        round: usize,
        total_rounds: usize,
        remaining: Duration,
    },
    Lobby {
        remaining: Duration,
    },
}

#[component]
pub(crate) fn Root(phase: GameState, show: bool) -> Option<Element> {
    if !show {
        return None;
    }

    let text = match phase {
        GameState::Idle => "No game in progress".white(),
        GameState::TrackRotation { combo } => {
            format!(
                "Loading track {}",
                combo.track(),
            )
            .white()
        },
        GameState::Round { round, combo, remaining } => {
            let seconds = remaining.as_secs() % 60;
            let minutes = (remaining.as_secs() / 60) % 60;
            format!(
                "Round {}/?? · {:02}:{:02} remaining",
                round, minutes, seconds
            )
            .white()
        },
        GameState::Lobby { combo } => {
            let seconds = remaining.as_secs() % 60;
            let minutes = (remaining.as_secs() / 60) % 60;
            format!("Lobby · {:02}:{:02} remaining", minutes, seconds).white()
        },
        GameState::Victory => {
            format!("Victory. Todo")
        },
        GameState::Exit => {
            unreachable!()
        }
    };

    let interface = cx
        .container()
        .h(150.0)
        .w(200.0)
        .flex()
        .flex_col()
        .with_child(cx.component::<Topbar>(TopbarProps { text }))
        .with_child(cx.component::<Motd>(MotdProps {
            text: WELCOME.to_owned(),
            what: 1,
        }));

    Some(interface)
}
