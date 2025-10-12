use std::collections::HashMap;

use insim::core::string::colours::Colourify;
use kitcar::ui::{
    wrap_text, Component, ComponentBehaviour, ComponentResult, Element, ElementKey, InstanceIdPool,
    Styled,
};

pub struct Motd {
    instance_id: u32,
    show: bool,
}

impl ComponentBehaviour for Motd {
    fn instance_id(&self) -> u32 {
        self.instance_id
    }

    fn children_mut(&mut self) -> Option<HashMap<u32, &mut dyn ComponentBehaviour>> {
        None
    }

    fn on_click(&mut self, click_id: &ElementKey) -> ComponentResult {
        if click_id.key == "motd_close" {
            self.show = false;
        }

        ComponentResult::default().render()
    }
}

impl Component for Motd {
    type Props = bool;

    fn mount(instance_ids: &mut InstanceIdPool, _props: Self::Props) -> Self {
        Self {
            instance_id: instance_ids.next(),
            show: true,
        }
    }

    fn update(&mut self, props: Self::Props) -> ComponentResult {
        self.show = props;
        ComponentResult::default().render()
    }

    fn render(&self) -> Option<kitcar::ui::Element> {
        if !self.show {
            return None;
        }

        // FIXME: we need a generic wrapped text component?
        let text: Vec<Element> = wrap_text(
            "Welcome drivers!
Forget being the fastest, the goal is to be the most precise. Finish in as close to 20secs as possible!
Full contact is allowed.
Just remember: Don't be a dick. We're all here to have fun!",
            5,
            78
        ).enumerate().map(|(i, line)| {
            Element::button(self.instance_id, &format!("motd_text_{}", i), line).h(5.).text_align_start()
        }).collect();

        if text.is_empty() {
            return None;
        }

        Some(
            Element::container().flex().flex_grow(1.0).with_child(
                Element::button(self.instance_id, "motd", "")
                    .flex()
                    .flex_col()
                    .w(80.)
                    .p(1.)
                    .light()
                    .my_auto()
                    .mx_auto()
                    .with_child(
                        Element::button(self.instance_id, "motd_inner", "")
                            .flex()
                            .flex_col()
                            .dark()
                            .p(1.)
                            .with_children(text),
                    )
                    .with_child(
                        Element::button(self.instance_id, "motd_close", &"Got it!".light_green())
                            .mt(2.)
                            .h(5.)
                            .green()
                            .dark()
                            .clickable(),
                    ),
            ),
        )
    }
}
