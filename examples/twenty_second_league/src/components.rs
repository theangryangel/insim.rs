use std::time::Duration;

use insim::insim::BtnStyle;
use kitcar::ui::{
    components::{button, fullscreen},
    node::UINode,
};

pub(crate) fn countdown(remaining: Duration) -> UINode {
    let seconds = remaining.as_secs() % 60;
    let minutes = (remaining.as_secs() / 60) % 60;

    fullscreen()
        .height(150.0)
        .display_flex()
        .flex_direction_column()
        .align_items_flex_start()
        .justify_content_flex_start()
        .padding(20.0)
        .with_child(
            kitcar::ui::node::UINode::rendered(BtnStyle::default().dark(), "", 1.into())
                .display_block()
                .position_relative()
                .padding(1.0)
                .with_children([
                    button("Welcome to ^120sl^8, game starts in".into(), 2.into())
                        .width(35.0)
                        .height(5.0),
                    button(format!("{:02}:{:02}", minutes, seconds).into(), 3.into())
                        .width(35.0)
                        .height(15.0),
                ]),
        )
}
