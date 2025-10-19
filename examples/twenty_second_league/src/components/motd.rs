use insim::core::string::colours::Colourify;
use kitcar::ui::{Component, Element, Scope, component, wrap_text};

use crate::components::textbox::{Textbox, TextboxProps};

#[component]
pub fn Motd(text: String, what: u8) -> Option<Element> {
    let show = cx.use_state(|| true);

    if !show.get() {
        cx.use_chat("!rules".to_string(), {
            println!("Adding chat/");
            let show = show.clone();
            move || {
                show.set(true);
            }
        });

        return None;
    }

    // FIXME: we need a generic wrapped text component?
    let wrapped_text: Vec<Element> = wrap_text(&text, 5, 78, 100)
        .enumerate()
        .map(|(_i, line)| {
            cx.button(line.to_string().black())
                .light()
                .h(5.)
                .text_align_start()
        })
        .collect();

    if wrapped_text.is_empty() {
        return None;
    }

    Some(
        cx.container()
            .flex()
            .flex_col()
            .my_auto()
            .mx_auto()
            .w(80.)
            .with_children(wrapped_text)
            .with_child(
                cx.button("Got it!".light_green())
                    .mt(2.)
                    .h(5.)
                    .green()
                    .light()
                    .on_click(Some(Box::new(move || {
                        println!("I GOT CLICKED! {:?}", what);
                        show.set(false);
                    }))),
            )
            .with_child(cx.container().mx_auto().with_child(cx.component::<Textbox>(
                TextboxProps {
                    text: text,
                    width: 80,
                    rows: 3,
                    row_height: 5,
                },
            ))),
    )
}
