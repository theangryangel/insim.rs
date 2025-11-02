use insim::core::string::colours::Colourify;
use kitcar::{
    chat::Parse,
    ui::{Element, Scope, component, wrap_text},
};

use crate::{
    chat::MyChatCommands,
    components::textbox::{Textbox, TextboxProps},
};

#[component]
pub fn Motd(text: String) -> Option<Element> {
    let show = cx.use_state(|| true);

    if !show.get() {
        cx.use_chat({
            let show = show.clone();
            move |input: &str| -> bool {
                if let Ok(MyChatCommands::Rules | MyChatCommands::Motd) =
                    MyChatCommands::parse(input)
                {
                    show.set(true);
                    true
                } else {
                    false
                }
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
        cx.background()
            .light()
            .flex()
            .flex_col()
            .my_auto()
            .mx_auto()
            .p(1.)
            .w(80.)
            .with_child(
                cx.button("Welcome to the Cadence Cup!".white())
                    .h(5.)
                    .mb(2.)
                    .text_align_start(),
            )
            .with_child(cx.container().mx_auto().with_child(cx.component::<Textbox>(
                TextboxProps {
                    text: text,
                    width: 80,
                    rows: 3,
                    row_height: 5,
                },
            )))
            .with_child(
                cx.button("Got it!".light_green())
                    .mt(2.)
                    .h(5.)
                    .green()
                    .dark()
                    .on_click(Some(Box::new(move || {
                        show.set(false);
                    }))),
            ),
    )
}
