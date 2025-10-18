use insim::core::string::colours::Colourify;
use kitcar::ui::{Component, Element, Scope, wrap_text};

use crate::components::textbox::{Textbox, TextboxProps};

#[derive(Clone, Debug)]
pub struct MotdProps {
    pub text: String,
    pub what: u8,
}

pub struct Motd;

impl Component for Motd {
    type Props = MotdProps;

    fn render(props: Self::Props, cx: &mut Scope) -> Option<kitcar::ui::Element> {
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
        let text: Vec<Element> = wrap_text(&props.text, 5, 78, 100)
            .enumerate()
            .map(|(_i, line)| {
                cx.button(line.to_string().black())
                    .light()
                    .h(5.)
                    .text_align_start()
            })
            .collect();

        if text.is_empty() {
            return None;
        }

        Some(
            cx.container()
                .flex()
                .flex_col()
                .my_auto()
                .mx_auto()
                .w(80.)
                .with_children(text)
                .with_child(
                    cx.button("Got it!".light_green())
                        .mt(2.)
                        .h(5.)
                        .green()
                        .light()
                        .on_click(Some(Box::new(move || {
                            println!("I GOT CLICKED! {:?}", props.what);
                            show.set(false);
                        }))),
                )
                .with_child(cx.container().mx_auto().with_child(cx.component::<Textbox>(
                    TextboxProps {
                        text: props.text.clone(),
                        width: 80,
                        rows: 3,
                        row_height: 5,
                    },
                ))),
        )
    }
}
