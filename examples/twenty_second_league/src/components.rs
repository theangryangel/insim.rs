use std::time::Duration;

use kitcar::ui::{
    components::{button, fullscreen},
    node::UINode,
};

pub(crate) fn countdown(remaining: Duration) -> UINode {
    let seconds = remaining.as_secs() % 60;
    let minutes = (remaining.as_secs() / 60) % 60;

    fullscreen()
        .height(150)
        .display_flex()
        .flex_direction_column()
        .justify_content_flex_start()
        .with_children([
            button("".into(), 4.into())
                .width(150)
                .height(150)
                .position_absolute()
                .dark(),
            button("Welcome to ^120sl^8, game starts in".into(), 2.into())
                .width(35)
                .height(5)
                .dark(),
            button(format!("{:02}:{:02}", minutes, seconds).into(), 3.into())
                .width(35)
                .height(15)
                .dark(),
        ])
}
