use std::collections::HashMap;

use insim::core::string::colours::Colourify;
use kitcar::ui::{wrap_text, Component, Element, Styled};

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

pub struct Motd {
    show: bool,
    what: usize,
}

impl Component for Motd {
    type Props = MotdProps;

    fn new(props: Self::Props) -> Self {
        Self {
            show: props.show,
            what: props.what,
        }
    }

    fn render(&self) -> Option<kitcar::ui::Element> {
        if !self.show {
            return None;
        }

        // FIXME: we need a generic wrapped text component?
        let text: Vec<Element> = wrap_text(WELCOME, 5, 78, 100)
            .enumerate()
            .map(|(_i, line)| Element::button(line).h(5.).text_align_start())
            .collect();

        if text.is_empty() {
            return None;
        }

        let what = self.what;

        Some(
            Element::container()
                .flex()
                .flex_col()
                .flex_grow(1.0)
                .with_child(
                    Element::button("")
                        .flex()
                        .flex_col()
                        .w(80.)
                        .p(1.)
                        .light()
                        .my_auto()
                        .mx_auto()
                        .with_child(
                            Element::button("")
                                .flex()
                                .flex_col()
                                .dark()
                                .p(1.)
                                .with_children(text),
                        )
                        .with_child(
                            Element::button(&"Got it!".light_green())
                                .mt(2.)
                                .h(5.)
                                .green()
                                .dark()
                                .on_click(Some(Box::new(move || {
                                    println!("I GOT CLICKED! {:?}", what);
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
