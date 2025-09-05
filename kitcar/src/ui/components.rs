//! Some basic reusable UI components

use std::borrow::Cow;

use insim::insim::BtnStyle;
use taffy::{
    prelude::length, style_helpers::max_content, AlignContent, AlignItems, Display, FlexDirection,
    Size, Style,
};

use crate::ui::node::{UINode, UINodeKey};

/// A primary button
pub fn primary_button(text: Cow<'static, str>, key: UINodeKey) -> UINode {
    UINode::Rendered {
        layout: Style {
            ..Default::default()
        },
        style: BtnStyle::default(),
        children: vec![],
        text,
        key,
    }
}

/// A basic button
pub fn basic(text: Cow<'static, str>, width: u8, height: u8, key: UINodeKey) -> UINode {
    UINode::Rendered {
        layout: Style {
            size: Size {
                width: length(width),
                height: length(height),
            },
            ..Default::default()
        },
        style: BtnStyle::default().dark().white().align_left(),
        text,
        key,
        children: vec![],
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

/// Create a "vstack" layout
pub fn vstack(children: Vec<UINode>) -> UINode {
    UINode::Unrendered {
        layout: Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: Some(AlignContent::FlexStart),
            align_items: Some(AlignItems::FlexStart),
            padding: length(20.0),
            size: Size {
                width: length(200.0),
                height: length(150.0),
            },
            ..Default::default()
        },
        children,
    }
}

/// Background
pub fn background(children: Vec<UINode>, key: UINodeKey) -> UINode {
    UINode::Rendered {
        layout: Style {
            size: Size {
                width: length(10.0),
                height: length(10.0),
            },
            padding: length(1.0),
            ..Default::default()
        },
        style: BtnStyle::default().dark(),
        text: "".into(),
        key,
        children,
    }
}
