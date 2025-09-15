//! Some basic reusable UI components

use std::borrow::Cow;

use insim::insim::BtnStyle;

use crate::ui::{
    node::{UINode, UINodeKey},
    style::Style,
};

/// Fullscreen
pub fn fullscreen() -> UINode {
    UINode::unrendered().width(200).height(200)
}

/// Create a "vstack" layout
pub fn vstack(children: Vec<UINode>) -> UINode {
    UINode::unrendered()
        .display_flex()
        .flex_direction_column()
        .with_children(children)
}

/// Create a "hstack" layout
pub fn hstack(children: Vec<UINode>) -> UINode {
    UINode::unrendered()
        .display_flex()
        .flex_direction_row()
        .justify_content_flex_start()
        .align_items_flex_start()
        .with_children(children)
}

/// A basic button
pub fn button(text: Cow<'static, str>, key: UINodeKey) -> UINode {
    UINode::rendered(text, key)
}
