use std::time::Duration;

use insim::core::string::colours::Colourify;
use kitcar::ui::{Element, Scope, component};

const WELCOME: &str = "Welcome drivers!
Forget being the fastest, the goal is to be the most precise. Finish in as close to 20secs as possible!
Full contact is allowed.
Just remember: Don't be a dick. We're all here to have fun!";

use crate::{
    ROUNDS_PER_GAME,
    components::{
        motd::{Motd, MotdProps},
        topbar::{Topbar, TopbarProps},
    },
};

#[derive(Debug, Clone)]
pub enum RootPhase {
    Idle,
    Game { round: usize, remaining: Duration },
    Victory,
}

#[component]
pub(crate) fn Root(phase: RootPhase, show: bool) -> Option<Element> {
    if !show {
        return None;
    }

    let text = match phase {
        RootPhase::Idle => "No game in progress".white(),
        RootPhase::Game { round, remaining } => {
            let seconds = remaining.as_secs() % 60;
            let minutes = (remaining.as_secs() / 60) % 60;
            format!(
                "Round {}/{} · {:02}:{:02} remaining",
                round, ROUNDS_PER_GAME, minutes, seconds
            )
            .white()
        },
        RootPhase::Victory => "Victory!".white(),
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
