use std::collections::HashMap;

use insim::core::string::colours::Colourify;
use kitcar::ui::{Component, Element, Styled};

use crate::{
    components::motd::{Motd, MotdProps},
    Phase, ROUNDS_PER_GAME,
};

#[derive(Debug, Clone)]
pub(crate) struct RootProps {
    pub phase: Phase,
    pub show: bool,
}

pub(crate) struct Root {
    pub phase: Phase,
    pub show: bool,
}

impl Component for Root {
    type Props = RootProps;

    fn new(props: Self::Props) -> Self {
        Self {
            phase: props.phase,
            show: props.show,
        }
    }

    fn render(&self) -> Option<Element> {
        let text = match self.phase {
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

        let interface = Element::container()
            .h(150.0)
            .w(200.0)
            .flex()
            .flex_col()
            .with_child(topbar(&text))
            .with_child(Element::Component(Box::new(Motd::new(MotdProps {
                show: true,
                what: 1,
            }))))
            .with_child(Element::Component(Box::new(Motd::new(MotdProps {
                show: true,
                what: 2,
            }))));

        Some(interface)
    }
}

// A not-component-component. We're just using this to make the Root component a bit more readable.
// We're going to assume that we can safely reuse the instance_id
pub(crate) fn topbar(text: &str) -> Element {
    // top bar
    Element::container()
        .flex()
        .flex_row()
        .justify_center()
        .with_child(
            Element::button(&format!(
                "{} {} {}",
                "Welcome to the".white(),
                "20".red(),
                "second league".white()
            ))
            .w(38.)
            .h(5.)
            .dark(),
        )
        .with_child(Element::button(&text).w(33.).h(5.).dark())
}
