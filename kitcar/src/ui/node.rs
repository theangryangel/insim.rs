//! UI Node and associated items

#![allow(missing_docs)]

use std::{
    borrow::Cow,
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
    ops::{Deref, DerefMut},
};

use insim::insim::{BtnStyle, BtnStyleColour, BtnStyleFlags};

use crate::ui::style::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// Unique key for a UINode.
/// Allows the Renderer to only send minimal updates to LFS.
pub struct UINodeKey(pub u8);

impl fmt::Display for UINodeKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for UINodeKey {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UINodeKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u8> for UINodeKey {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug)]
/// UINode
pub enum UINode {
    /// Rendered items generate buttons in LFS
    Rendered {
        /// Requested layout
        // XXX: taffy Style is not Send. So.. I guess we need to have our own :(
        // https://github.com/DioxusLabs/taffy/issues/823
        style: Style,
        /// Button Text
        text: Cow<'static, str>,
        /// *Your* ClickId - the ClickId sent to LFS will be assigned
        key: UINodeKey,
    },
    /// Unrendered items are just used to help the layout generation
    Unrendered {
        /// Requested layout
        // XXX: see above
        style: Style,
        /// Child nodes
        children: Vec<Self>,
    },
}

impl UINode {
    pub(crate) fn text(&self) -> Option<Cow<'static, str>> {
        match self {
            UINode::Rendered { text, .. } => Some(text.clone()),
            UINode::Unrendered { .. } => None,
        }
    }

    pub(crate) fn children(&self) -> Option<&[Self]> {
        match self {
            Self::Unrendered { children, .. } => Some(children),
            Self::Rendered { .. } => None,
        }
    }

    pub(crate) fn style(&self) -> &Style {
        match self {
            UINode::Rendered { style, .. } => style,
            UINode::Unrendered { style, .. } => style,
        }
    }

    /// Creates a new rendered node.
    pub fn rendered(text: impl Into<Cow<'static, str>>, key: UINodeKey) -> Self {
        Self::Rendered {
            style: Style::default(),
            text: text.into(),
            key,
        }
    }

    /// Creates a new unrendered node.
    pub fn unrendered() -> Self {
        Self::Unrendered {
            style: Style::default(),
            children: Vec::new(),
        }
    }

    // Child management
    /// Adds a single child.
    pub fn with_child(mut self, child: UINode) -> Self {
        if let UINode::Unrendered {
            ref mut children, ..
        } = self
        {
            children.push(child);
        }
        self
    }

    /// Adds a list of children.
    pub fn with_children<I>(mut self, new_children: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        if let UINode::Unrendered {
            ref mut children, ..
        } = self
        {
            children.extend(new_children);
        }
        self
    }

    fn style_mut(&mut self) -> &mut Style {
        match self {
            UINode::Rendered { style, .. } => style,
            UINode::Unrendered { style, .. } => style,
        }
    }

    // Display

    pub fn display_flex(mut self) -> Self {
        self.style_mut().display = Display::Flex;
        self
    }

    pub fn display_block(mut self) -> Self {
        self.style_mut().display = Display::Block;
        self
    }

    pub fn display_grid(mut self) -> Self {
        self.style_mut().display = Display::Grid;
        self
    }

    // Position

    pub fn position_relative(mut self) -> Self {
        self.style_mut().position = Position::Relative;
        self
    }

    pub fn position_absolute(mut self) -> Self {
        self.style_mut().position = Position::Absolute;
        self
    }

    pub fn overflow_visible(mut self) -> Self {
        self.style_mut().overflow = Some(Overflow::Visible);
        self
    }

    pub fn overflow_clip(mut self) -> Self {
        self.style_mut().overflow = Some(Overflow::Clip);
        self
    }

    pub fn overflow_hidden(mut self) -> Self {
        self.style_mut().overflow = Some(Overflow::Hidden);
        self
    }

    pub fn overflow_scroll(mut self) -> Self {
        self.style_mut().overflow = Some(Overflow::Scroll);
        self
    }

    pub fn overflow_unset(mut self) -> Self {
        self.style_mut().overflow = None;
        self
    }

    // Flex

    pub fn flex_direction_column(mut self) -> Self {
        self.style_mut().flex_direction = Some(FlexDirection::Column);
        self
    }

    pub fn flex_direction_column_reverse(mut self) -> Self {
        self.style_mut().flex_direction = Some(FlexDirection::ColumnReverse);
        self
    }

    pub fn flex_direction_row(mut self) -> Self {
        self.style_mut().flex_direction = Some(FlexDirection::Row);
        self
    }

    pub fn flex_direction_row_reverse(mut self) -> Self {
        self.style_mut().flex_direction = Some(FlexDirection::RowReverse);
        self
    }

    pub fn flex_direction_unset(mut self) -> Self {
        self.style_mut().flex_direction = None;
        self
    }

    pub fn flex_nowrap(mut self) -> Self {
        self.style_mut().flex_wrap = Some(FlexWrap::NoWrap);
        self
    }

    pub fn flex_wrap(mut self) -> Self {
        self.style_mut().flex_wrap = Some(FlexWrap::Wrap);
        self
    }

    pub fn flex_wrap_reverse(mut self) -> Self {
        self.style_mut().flex_wrap = Some(FlexWrap::WrapReverse);
        self
    }

    pub fn flex_wrap_unset(mut self) -> Self {
        self.style_mut().flex_wrap = None;
        self
    }

    pub fn flex_grow<V: Into<Option<f32>>>(mut self, val: V) -> Self {
        self.style_mut().flex_grow = val.into();
        self
    }

    pub fn flex_shrink<V: Into<Option<f32>>>(mut self, val: V) -> Self {
        self.style_mut().flex_shrink = val.into();
        self
    }

    // align item

    pub fn align_items_start(mut self) -> Self {
        self.style_mut().align_items = Some(AlignItems::Start);
        self
    }

    pub fn align_items_end(mut self) -> Self {
        self.style_mut().align_items = Some(AlignItems::End);
        self
    }

    pub fn align_items_flex_start(mut self) -> Self {
        self.style_mut().align_items = Some(AlignItems::FlexStart);
        self
    }

    pub fn align_items_flex_end(mut self) -> Self {
        self.style_mut().align_items = Some(AlignItems::FlexEnd);
        self
    }

    pub fn align_items_center(mut self) -> Self {
        self.style_mut().align_items = Some(AlignItems::Center);
        self
    }

    pub fn align_items_baseline(mut self) -> Self {
        self.style_mut().align_items = Some(AlignItems::Baseline);
        self
    }

    pub fn align_items_stretch(mut self) -> Self {
        self.style_mut().align_items = Some(AlignItems::Stretch);
        self
    }

    // align self
    pub fn align_self_start(mut self) -> Self {
        self.style_mut().align_self = Some(AlignSelf::Start);
        self
    }

    pub fn align_self_end(mut self) -> Self {
        self.style_mut().align_self = Some(AlignSelf::End);
        self
    }

    pub fn align_self_flex_start(mut self) -> Self {
        self.style_mut().align_self = Some(AlignSelf::FlexStart);
        self
    }

    pub fn align_self_flex_end(mut self) -> Self {
        self.style_mut().align_self = Some(AlignSelf::FlexEnd);
        self
    }

    pub fn align_self_center(mut self) -> Self {
        self.style_mut().align_self = Some(AlignSelf::Center);
        self
    }

    pub fn align_self_baseline(mut self) -> Self {
        self.style_mut().align_self = Some(AlignSelf::Baseline);
        self
    }

    pub fn align_self_stretch(mut self) -> Self {
        self.style_mut().align_self = Some(AlignSelf::Stretch);
        self
    }

    // align content
    pub fn align_content_start(mut self) -> Self {
        self.style_mut().align_content = Some(AlignContent::Start);
        self
    }

    pub fn align_content_end(mut self) -> Self {
        self.style_mut().align_content = Some(AlignContent::End);
        self
    }

    pub fn align_content_flex_start(mut self) -> Self {
        self.style_mut().align_content = Some(AlignContent::FlexStart);
        self
    }

    pub fn align_content_flex_end(mut self) -> Self {
        self.style_mut().align_content = Some(AlignContent::FlexEnd);
        self
    }

    pub fn align_content_center(mut self) -> Self {
        self.style_mut().align_content = Some(AlignContent::Center);
        self
    }

    pub fn align_content_stretch(mut self) -> Self {
        self.style_mut().align_content = Some(AlignContent::Stretch);
        self
    }

    pub fn align_content_space_between(mut self) -> Self {
        self.style_mut().align_content = Some(AlignContent::SpaceBetween);
        self
    }

    pub fn align_content_space_evenly(mut self) -> Self {
        self.style_mut().align_content = Some(AlignContent::SpaceEvenly);
        self
    }

    pub fn align_content_space_around(mut self) -> Self {
        self.style_mut().align_content = Some(AlignContent::SpaceAround);
        self
    }

    // justify content

    pub fn justify_content_start(mut self) -> Self {
        self.style_mut().justify_content = Some(JustifyContent::Start);
        self
    }

    pub fn justify_content_end(mut self) -> Self {
        self.style_mut().justify_content = Some(JustifyContent::End);
        self
    }

    pub fn justify_content_flex_start(mut self) -> Self {
        self.style_mut().justify_content = Some(JustifyContent::FlexStart);
        self
    }

    pub fn justify_content_flex_end(mut self) -> Self {
        self.style_mut().justify_content = Some(JustifyContent::FlexEnd);
        self
    }

    pub fn justify_content_center(mut self) -> Self {
        self.style_mut().justify_content = Some(JustifyContent::Center);
        self
    }

    pub fn justify_content_stretch(mut self) -> Self {
        self.style_mut().justify_content = Some(JustifyContent::Stretch);
        self
    }

    pub fn justify_content_space_between(mut self) -> Self {
        self.style_mut().justify_content = Some(JustifyContent::SpaceBetween);
        self
    }

    pub fn justify_content_space_evenly(mut self) -> Self {
        self.style_mut().justify_content = Some(JustifyContent::SpaceEvenly);
        self
    }

    pub fn justify_content_space_around(mut self) -> Self {
        self.style_mut().justify_content = Some(JustifyContent::SpaceAround);
        self
    }

    // bstyle
    pub fn dark(mut self) -> Self {
        let bstyle = self.style_mut().bstyle.clone().dark();
        self.style_mut().bstyle = bstyle;

        self
    }
}

macro_rules! generate_style_fns {
    ($property:ident, $type_name:ident) => {
        paste::paste! {
            impl UINode {
                /// Sets the `
                #[doc = stringify!($property)]
                /// ` property to `Auto`.
                pub fn [< $property _auto >](mut self) -> Self {
                    self.style_mut().$property = $type_name::Auto;
                    self
                }

                /// Sets the `
                #[doc = stringify!($property)]
                /// ` property to a fixed length or percentage, depending if passed a float or unsigned int.
                pub fn $property<V: Into<$type_name>>(mut self, size: V) -> Self {
                    self.style_mut().$property = size.into();
                    self
                }

                /// Resets the `
                #[doc = stringify!($property)]
                /// ` property to `Unset`.
                pub fn [< $property _unset >](mut self) -> Self {
                    self.style_mut().$property = $type_name::Unset;
                    self
                }
            }
        }
    };
}

macro_rules! generate_style_optional_fns {
    ($property:ident, $type_name:ident) => {
        paste::paste! {
            impl UINode {
                /// Sets the `
                #[doc = stringify!($property)]
                /// ` property to None.
                pub fn [< $property _none >](mut self) -> Self {
                    self.style_mut().$property = None;
                    self
                }

                /// Sets the `
                #[doc = stringify!($property)]
                /// ` property to a fixed length or percentage, depending if passed a float or unsigned int.
                pub fn $property<V: Into<$type_name>>(mut self, size: V) -> Self {
                    self.style_mut().$property = Some(size.into());
                    self
                }
            }
        }
    };
}

generate_style_fns!(inset, Inset);
generate_style_fns!(width, Size);
generate_style_fns!(height, Size);

generate_style_fns!(margin, Margin);
generate_style_optional_fns!(margin_left, Margin);
generate_style_optional_fns!(margin_right, Margin);
generate_style_optional_fns!(margin_top, Margin);
generate_style_optional_fns!(margin_bottom, Margin);

generate_style_fns!(padding, Padding);
generate_style_optional_fns!(padding_left, Padding);
generate_style_optional_fns!(padding_right, Padding);
generate_style_optional_fns!(padding_top, Padding);
generate_style_optional_fns!(padding_bottom, Padding);

generate_style_fns!(gap, Gap);
generate_style_optional_fns!(column_gap, Gap);
generate_style_optional_fns!(row_gap, Gap);

generate_style_optional_fns!(flex_basis, FlexBasis);
