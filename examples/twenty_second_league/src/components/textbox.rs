use insim::core::string::colours::Colourify;
use kitcar::ui::{Component, Element, Scope, wrap_text};

#[derive(Clone, Debug)]
pub struct TextboxProps {
    pub text: String,
    pub rows: u8,
    pub width: u8,
    pub row_height: u8,
}

pub struct Textbox;

impl Component for Textbox {
    type Props = TextboxProps;

    fn render(props: Self::Props, cx: &mut Scope) -> Option<kitcar::ui::Element> {
        if props.text.is_empty() {
            return None;
        }
        let offset = cx.use_state(|| 0 as usize);

        let wrapped: Vec<&str> = wrap_text(&props.text, props.row_height, props.width, 100)
            .into_iter()
            .collect();
        let end = offset.get() + props.rows as usize;
        let slice_end = std::cmp::min(end, wrapped.len());
        let offset_by = std::cmp::min(offset.get(), slice_end);

        let collected: Vec<Element> = (&wrapped[offset_by..slice_end])
            .iter()
            .map(|f| {
                cx.button(f.black())
                    .light()
                    .w(props.width as f32 - 6.)
                    .h(props.row_height as f32)
                    .text_align_start()
            })
            .collect();

        Some(
            cx.container()
                .flex()
                .flex_row()
                .p(1.)
                .with_child(
                    cx.container()
                        .p(1.)
                        .flex()
                        .flex_col()
                        .with_children(collected),
                )
                .with_child(
                    cx.container()
                        .flex()
                        .flex_col()
                        .flex_grow(1.)
                        .with_child(cx.button("▲".black()).light().w(5.).h(5.).on_click({
                            let offset = offset.clone();

                            Some(Box::new(move || {
                                let next = offset.get().saturating_sub(1);

                                offset.set(next);
                                println!("Up was clicked!");
                            }))
                        }))
                        .with_child(cx.container().flex().flex_grow(1.))
                        .with_child(cx.button("▼".black()).light().w(5.).h(5.).on_click({
                            let offset = offset.clone();

                            Some(Box::new(move || {
                                let next = std::cmp::min(
                                    offset.get() + 1,
                                    slice_end + 1 - props.rows as usize,
                                );

                                offset.set(next);
                                println!("Down was clicked!");
                            }))
                        })),
                ),
        )
    }
}
