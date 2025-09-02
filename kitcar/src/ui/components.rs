//! Some basic reusable UI components

use std::borrow::Cow;

use insim::insim::BtnStyle;
use taffy::{prelude::length, AlignItems, FlexDirection, Size, Style};

use crate::ui::node::{UINode, UINodeKey};

/// Create a basic button
pub fn primary_button(text: Cow<'static, str>, key: UINodeKey) -> UINode {
    UINode::Rendered {
        layout: Style {
            size: Size {
                width: length(120.0),
                height: length(40.0),
            },
            ..Default::default()
        },
        style: BtnStyle::default(),
        text,
        key,
    }
}

/// Create a "page" layout
pub fn page_layout(children: Vec<UINode>) -> UINode {
    UINode::Unrendered {
        layout: Style {
            flex_direction: FlexDirection::Column,
            align_items: Some(AlignItems::Center),
            ..Default::default()
        },
        children,
    }
}
