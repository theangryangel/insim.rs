use std::collections::HashMap;

use insim::core::string::colours::Colourify;
use kitcar::ui::{Component, ComponentBehaviour, ComponentResult, Element, InstanceIdPool, Styled};

use crate::{components::motd::Motd, Phase, ROUNDS_PER_GAME};

pub(crate) struct Root {
    instance_id: u32,
    phase: Phase,
    show: bool,
    motd: super::motd::Motd,
    motd2: super::motd::Motd,
}

impl ComponentBehaviour for Root {
    fn instance_id(&self) -> u32 {
        self.instance_id
    }

    fn on_mso(&mut self, mso: &insim::insim::Mso) -> ComponentResult {
        match mso.msg_from_textstart() {
            "!toggle" => {
                self.show = !self.show;
                ComponentResult::default().render()
            },
            "!rules" => {
                let _ = self.motd.update(true);
                self.motd2.update(true)
            },
            _ => ComponentResult::default(),
        }
    }

    fn children_mut(&mut self) -> Option<HashMap<u32, &mut dyn ComponentBehaviour>> {
        Some(HashMap::from([
            (
                self.motd.instance_id(),
                &mut self.motd as &mut dyn ComponentBehaviour,
            ),
            (
                self.motd2.instance_id(),
                &mut self.motd2 as &mut dyn ComponentBehaviour,
            ),
        ]))
    }
}

impl Component for Root {
    type Props = Phase;

    fn mount(instance_ids: &mut InstanceIdPool, props: Self::Props) -> Self {
        Self {
            instance_id: instance_ids.next(),
            phase: props,
            show: true,
            motd: Motd::mount(instance_ids, true),
            motd2: Motd::mount(instance_ids, true),
        }
    }

    fn update(&mut self, props: Self::Props) -> ComponentResult {
        if self.phase == props {
            self.phase = props;
            ComponentResult::default().render()
        } else {
            ComponentResult::default()
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
            .with_child_if(topbar(self.instance_id, &text), self.show)
            .try_with_child(self.motd.render())
            .try_with_child(self.motd2.render());

        Some(interface)
    }
}

// A not-component-component. We're just using this to make the Root component a bit more readable.
// We're going to assume that we can safely reuse the instance_id
pub(crate) fn topbar(instance_id: u32, text: &str) -> Element {
    // top bar
    Element::container()
        .flex()
        .flex_row()
        .justify_center()
        .with_child(
            Element::button(
                instance_id,
                "welcome",
                &format!(
                    "{} {} {}",
                    "Welcome to the".white(),
                    "20".red(),
                    "second league".white()
                ),
            )
            .w(38.)
            .h(5.)
            .dark(),
        )
        .with_child(
            Element::button(
                instance_id,
                "countdown",
                // ,
                &text,
            )
            .w(33.)
            .h(5.)
            .dark(),
        )
}
