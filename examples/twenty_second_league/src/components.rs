use std::time::Duration;

use kitcar::ui::{Element, Styled};

pub(crate) fn countdown(remaining: &Duration) -> Option<Element> {
    let seconds = remaining.as_secs() % 60;
    let minutes = (remaining.as_secs() / 60) % 60;

    Some(
        Element::container()
            .h(150.)
            .w(200.)
            .p(20.)
            .with_child(
                Element::button("welcome", "Welcome to ^120sl^8, game starts in")
                    .w(35.)
                    .h(5.),
            )
            .with_child(
                Element::button("countdown", &format!("{:02}:{:02}", minutes, seconds))
                    .w(35.)
                    .h(15.),
            ),
    )
}
