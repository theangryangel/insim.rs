use insim::core::string::colours::Colourify;
use kitcar::ui::{
    Component, ComponentHandler, ComponentResult, Element, ElementKey, InstanceIdPool, Styled,
};

use crate::{components::motd::Motd, Phase, ROUNDS_PER_GAME};

pub(crate) struct Root {
    instance_id: u32,
    phase: Phase,
    show: bool,
    motd: super::motd::Motd,
}

impl ComponentHandler for Root {
    fn instance_id(&self) -> u32 {
        self.instance_id
    }

    fn on_click(&mut self, _click_id: &ElementKey) -> ComponentResult {
        ComponentResult::default()
    }

    fn on_mso(&mut self, mso: &insim::insim::Mso) -> ComponentResult {
        match mso.msg_from_textstart() {
            "!toggle" => {
                self.show = !self.show;
                ComponentResult::default().render()
            },
            "!rules" => self.motd.update(true),
            _ => ComponentResult::default(),
        }
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
            .with_child(topbar(self.instance_id, &text))
            .try_with_child(self.motd.render());

        Some(interface)
    }
}

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
