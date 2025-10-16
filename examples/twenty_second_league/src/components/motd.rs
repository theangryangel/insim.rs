use insim::core::string::colours::Colourify;
use kitcar::ui::{wrap_text, Component, Element, Scope};

// use crate::components::textbox::Textbox;

const WELCOME: &str = "Welcome drivers!
Forget being the fastest, the goal is to be the most precise. Finish in as close to 20secs as possible!
Full contact is allowed.
Just remember: Don't be a dick. We're all here to have fun!";

#[derive(Clone, Debug)]
pub struct MotdProps {
    pub show: bool,
    pub what: usize,
}

pub struct Motd;

impl Component for Motd {
    type Props = MotdProps;

    fn render(props: Self::Props, cx: &mut Scope) -> Option<kitcar::ui::Element> {
        if !props.show {
            return None;
        }

        // FIXME: we need a generic wrapped text component?
        let text: Vec<Element> = wrap_text(WELCOME, 5, 78, 100)
            .enumerate()
            .map(|(_i, line)| cx.button(line.into()).h(5.).text_align_start())
            .collect();

        if text.is_empty() {
            return None;
        }

        Some(
            cx.container().flex().flex_col().flex_grow(1.0).with_child(
                cx.button("".into())
                    .flex()
                    .flex_col()
                    .w(80.)
                    .p(1.)
                    .light()
                    .my_auto()
                    .mx_auto()
                    .with_child(
                        cx.button("".into())
                            .flex()
                            .flex_col()
                            .dark()
                            .p(1.)
                            .with_children(text),
                    )
                    .with_child(
                        cx.button("Got it!".light_green())
                            .mt(2.)
                            .h(5.)
                            .green()
                            .dark()
                            .on_click(Some(Box::new(move || {
                                println!("I GOT CLICKED! {:?}", props.what);
                            }))),
                    ),
            ), // .with_child(
               //     Element::container()
               //         .mx_auto()
               //         .with_children(
               //             Element::Component(Textbox)
               //         ),
               // ),
        )
    }
}
