use insim::core::string::colours::Colourify;
use kitcar::ui::{Component, Element, Scope};

use crate::{
    components::{
        motd::{Motd, MotdProps},
        topbar::Topbar,
    },
    Phase, ROUNDS_PER_GAME,
};

#[derive(Debug, Clone)]
pub(crate) struct RootProps {
    pub phase: Phase,
    pub show: bool,
}

pub(crate) struct Root;

impl Component for Root {
    type Props = RootProps;

    fn render(props: Self::Props, cx: &mut Scope) -> Option<Element> {
        if !props.show {
            return None;
        }

        let text = match props.phase {
            Phase::Idle => "No game in progress".white(),
            Phase::Game { round, remaining } => {
                let seconds = remaining.as_secs() % 60;
                let minutes = (remaining.as_secs() / 60) % 60;
                format!(
                    "Round {}/{} · {:02}:{:02} remaining",
                    round, ROUNDS_PER_GAME, minutes, seconds
                )
                .white()
            },
            Phase::Victory => "Victory!".white(),
        };

        let interface = cx
            .container()
            .h(150.0)
            .w(200.0)
            .flex()
            .flex_col()
            .with_child(cx.component::<Topbar>(text))
            .with_child(cx.component::<Motd>(MotdProps {
                show: true,
                what: 1,
            }))
            .with_child(cx.component::<Motd>(MotdProps {
                show: true,
                what: 2,
            }));

        Some(interface)
    }
}
