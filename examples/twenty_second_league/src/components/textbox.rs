use insim::core::string::colours::Colourify;
use kitcar::ui::{Element, Scope, component, wrap_text};

#[component]
pub(crate) fn Textbox(text: String, rows: u8, width: u8, row_height: u8) -> Option<Element> {
    if text.is_empty() {
        return None;
    }
    let offset = cx.use_state(|| 0 as usize);

    let wrapped: Vec<&str> = wrap_text(&text, row_height, width - 7, 100)
        .into_iter()
        .collect();
    let end = offset.get() + rows as usize;
    let slice_end = std::cmp::min(end, wrapped.len());
    let offset_by = std::cmp::min(offset.get(), slice_end);

    let collected: Vec<Element> = (&wrapped[offset_by..slice_end])
        .iter()
        .map(|f| {
            cx.button(f.white())
                .w(width as f32 - 7.)
                .h(row_height as f32)
                .text_align_start()
        })
        .collect();

    Some(
        cx.container()
            .flex()
            .flex_row()
            .with_child(
                cx.background()
                    .light()
                    .flex()
                    .flex_col()
                    .with_children(collected),
            )
            .with_child(
                cx.container()
                    .flex()
                    .flex_col()
                    .flex_grow(1.)
                    .with_child(cx.button("▲".white()).dark().w(4.).h(4.).on_click({
                        let offset = offset.clone();

                        Some(Box::new(move || {
                            let next = offset.get().saturating_sub(1);
                            offset.set(next);
                        }))
                    }))
                    .with_child(cx.container().flex().flex_grow(1.))
                    .with_child(cx.button("▼".white()).dark().w(4.).h(4.).on_click({
                        let offset = offset.clone();
                        let rows = rows as usize;
                        let len = wrapped.len();

                        Some(Box::new(move || {
                            // prevent over scroll
                            let next = std::cmp::min(offset.get() + 1, len.saturating_sub(rows));
                            offset.set(next);
                        }))
                    })),
            ),
    )
}
