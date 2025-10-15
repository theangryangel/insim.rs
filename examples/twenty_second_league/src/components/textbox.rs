use std::collections::HashMap;

use insim::core::string::colours::Colourify;
use kitcar::ui::{wrap_text, Component, Element, InstanceIdPool, Styled};

pub struct TextboxProps {
    text: String,
    rows: u8,
    width: u8,
    row_height: u8,
    offset: usize,
}

pub struct Textbox;

impl Component for Textbox {
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
                Element::button(&f.white())
                    .w(self.width as f32 - 6.)
                    .h(self.row_height as f32)
                    .text_align_start()
            })
            .collect();

        Some(
            Element::button("")
                .light()
                .flex()
                .flex_row()
                .p(1.)
                .with_child(
                    Element::button("")
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
                        .with_child(Element::button(&"▲".white()).dark().w(5.).h(5.).on_click(
                            Some(Box::new(|| {
                                println!("Up was clicked!");
                            })),
                        ))
                        .with_child(Element::container().flex().flex_grow(1.))
                        .with_child(Element::button(&"▼".white()).dark().w(5.).h(5.).on_click(
                            Some(Box::new(|| {
                                println!("Down was clicked!");
                            })),
                        )),
                ),
        )
    }
}
