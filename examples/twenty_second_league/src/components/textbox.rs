use std::collections::HashMap;

use insim::core::string::colours::Colourify;
use kitcar::ui::{
    wrap_text, Component, ComponentBehaviour, ComponentResult, Element, ElementKey, InstanceIdPool,
    Styled,
};

pub struct Textbox {
    instance_id: u32,
    text: String,
    rows: u8,
    width: u8,
    row_height: u8,
    offset: usize,
}

impl ComponentBehaviour for Textbox {
    fn instance_id(&self) -> u32 {
        self.instance_id
    }

    fn children_mut(&mut self) -> Option<HashMap<u32, &mut dyn ComponentBehaviour>> {
        None
    }

    fn on_click(&mut self, click_id: &ElementKey) -> ComponentResult {
        if click_id.key == "up" {
            self.offset = self.offset.saturating_sub(1);
        }

        if click_id.key == "down" {
            self.offset = self.offset.saturating_add(1);
        }

        ComponentResult::default().render()
    }
}

impl Component for Textbox {
    type Props = String;

    fn mount(instance_ids: &mut InstanceIdPool, props: Self::Props) -> Self {
        Self {
            instance_id: instance_ids.next(),
            text: props,
            width: 80,
            rows: 3,
            row_height: 5,
            offset: 0,
        }
    }

    fn update(&mut self, props: Self::Props) -> ComponentResult {
        self.text = props;
        ComponentResult::default().render()
    }

    fn render(&self) -> Option<kitcar::ui::Element> {
        if self.text.is_empty() {
            return None;
        }

        let wrapped: Vec<&str> = wrap_text(&self.text, self.row_height, self.width, 100)
            .into_iter()
            .collect();
        let end = self.offset as usize + self.rows as usize;
        let has_more = end < wrapped.len();
        let slice_end = std::cmp::min(end, wrapped.len());
        let offset = std::cmp::min(self.offset as usize, slice_end);

        let collected: Vec<Element> = (&wrapped[offset..slice_end])
            .iter()
            .enumerate()
            .map(|(i, f)| {
                Element::button(self.instance_id(), &format!("textarea_{}", i), &f.white())
                    .w(self.width as f32 - 6.)
                    .h(self.row_height as f32)
                    .text_align_start()
            })
            .collect();

        Some(
            Element::button(self.instance_id(), "outer", "")
                .light()
                .flex()
                .flex_row()
                .p(1.)
                .with_child(
                    Element::button(self.instance_id(), "bg", "")
                        .p(1.)
                        .dark()
                        .flex()
                        .flex_col()
                        .with_children(collected),
                )
                .with_child(
                    Element::container()
                        .flex()
                        .flex_col()
                        .flex_grow(1.)
                        .with_child(
                            Element::button(self.instance_id(), "up", &"▲".white())
                                .dark()
                                .w(5.)
                                .h(5.)
                                .clickable(self.offset > 0),
                        )
                        .with_child(Element::container().flex().flex_grow(1.))
                        .with_child(
                            Element::button(self.instance_id(), "down", &"▼".white())
                                .dark()
                                .w(5.)
                                .h(5.)
                                .clickable(has_more),
                        ),
                ),
        )
    }
}
