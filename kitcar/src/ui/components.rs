//! Some basic reusable UI components

use std::borrow::Cow;

use insim::insim::BtnStyle;
use taffy::{prelude::length, AlignContent, AlignItems, Display, FlexDirection, Size, Style};

use crate::ui::node::{UINode, UINodeKey};

/// Fullscreen
pub fn fullscreen() -> UINode {
    UINode::Unrendered {
        layout: Style {
            size: Size {
                width: length(200.0),
                height: length(200.0),
            },
            ..Default::default()
        },
        children: Vec::new(),
    }
}

/// Create a "vstack" layout
pub fn vstack(children: Vec<UINode>) -> UINode {
    UINode::Unrendered {
        layout: Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: Some(AlignContent::FlexStart),
            align_items: Some(AlignItems::FlexStart),
            ..Default::default()
        },
        children,
    }
}

/// Create a "hstack" layout
pub fn hstack(children: Vec<UINode>) -> UINode {
    UINode::Unrendered {
        layout: Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: Some(AlignContent::FlexStart),
            align_items: Some(AlignItems::FlexStart),
            ..Default::default()
        },
        children,
    }
}

/// A basic button
pub fn button(text: Cow<'static, str>, key: UINodeKey) -> UINode {
    UINode::Rendered {
        layout: Style::default(),
        style: BtnStyle::default(),
        text,
        key,
    }
}
