use std::time::Duration;

use kitcar::ui::{Element, Styled};

pub(crate) fn countdown(remaining: &Duration) -> Option<Element> {
    let seconds = remaining.as_secs() % 60;
    let minutes = (remaining.as_secs() / 60) % 60;

    Some(
        Element::container()
            // .h_auto()
            .h(200.)
            .w(200.)
            .flex()
            .flex_col()
            .with_child(
                Element::container()
                    .flex()
                    .flex_row()
                    .justify_center()
                    .with_child(
                        Element::button("welcome", "Welcome to ^120^8 second league")
                            .w(33.)
                            .h(5.),
                    )
                    .with_child(
                        Element::button(
                            "countdown",
                            &format!("Warmup. Game starts in {:02}:{:02}", minutes, seconds),
                        )
                        .w(33.)
                        .h(5.),
                    )
                    .with_child(
                        Element::button("round_info", "Round 1/20 · 1st · +22pts")
                            .h(5.)
                            .w(33.),
                    ),
            )
            .with_child(
                Element::container()
                    .flex()
                    .mt_auto()
                    .justify_start()
                    .with_child(
                        Element::button("plugin_info", "powered by insim.rs")
                            .h(5.)
                            .w(30.),
                    ),
            ),
    )
}
