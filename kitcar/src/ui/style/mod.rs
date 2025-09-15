//! A Send + Sync capable reimplementation of the parts of `taffy::Style` used in the project.

#![allow(missing_docs)]

use std::{fmt::Debug, hash::Hash};

use insim::insim::BtnStyle;
// Re-export stuff we can use
pub use taffy::style::{
    AlignContent, AlignItems, AlignSelf, Display, FlexDirection, FlexWrap, JustifyContent,
    Overflow, Position,
};

macro_rules! new_size_type {
    ($(#[$m:meta])* $name:ident, $intrepr:ty, $def:expr) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub enum $name {
            /// The default behavior.
            #[default]
            Unset,
            /// Automatically selects a suitable size.
            Auto,
            /// Sets an absolute value.
            Length($intrepr),
            /// Sets a percentage of the width or height of the parent.
            Percent(f32),
        }

        impl From<$intrepr> for $name {
            fn from(val: $intrepr) -> Self { Self::Length(val) }
        }

        impl From<f32> for $name {
            fn from(val: f32) -> Self { Self::Percent(val) }
        }

        impl From<$name> for taffy::LengthPercentageAuto {
            fn from(p: $name) -> Self {
                match p {
                    $name::Unset => $def.into(),
                    $name::Auto => taffy::LengthPercentageAuto::auto(),
                    $name::Length(l) => taffy::LengthPercentageAuto::length(l as _),
                    $name::Percent(p) => taffy::LengthPercentageAuto::percent(p / 100.0),
                }
            }
        }

        impl From<$name> for taffy::LengthPercentage {
            fn from(p: $name) -> Self {
                match p {
                    $name::Unset => $def.into(),
                    $name::Auto => <taffy::LengthPercentage as taffy::style_helpers::TaffyZero>::ZERO,
                    $name::Length(l) => taffy::LengthPercentage::length(l as _),
                    $name::Percent(p) => taffy::LengthPercentage::percent(p / 100.0),
                }
            }
        }

        impl From<$name> for taffy::Dimension {
            fn from(p: $name) -> Self {
                match p {
                    $name::Unset => $def.into(),
                    $name::Auto => taffy::Dimension::auto(),
                    $name::Length(l) => taffy::Dimension::length(l as _),
                    $name::Percent(p) => taffy::Dimension::percent(p / 100.0),
                }
            }
        }
    };
}

new_size_type!(
    /// Defines the area to reserve around the element's content, but outside the border.
    Margin,
    u8,
    Margin::Length(0)
);

new_size_type!(
    /// Defines a width or height of an element.
    Size,
    u8,
    Size::Auto
);

new_size_type!(
    /// Sets the position of a positioned element.
    Inset,
    u8,
    Size::Auto
);

new_size_type!(
    /// Defines the area to reserve around the element's content, but inside the border.
    Padding,
    u8,
    Padding::Length(0)
);

new_size_type!(
    /// Defines the gaps in between rows or columns of flex items.
    Gap,
    u8,
    Gap::Length(0)
);

new_size_type!(FlexBasis, u8, FlexBasis::Auto);

/// A `Send`-capable replacement for `taffy::style::Style`.
///
/// This struct can be safely sent across threads. It implements `From<SendableStyle>`
/// for `taffy::style::Style` to allow for easy conversion before passing it to Taffy.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Style {
    pub(crate) display: Display,
    pub(crate) position: Position,

    pub(crate) inset: Inset,
    pub(crate) inset_left: Option<Inset>,
    pub(crate) inset_right: Option<Inset>,
    pub(crate) inset_top: Option<Inset>,
    pub(crate) inset_bottom: Option<Inset>,

    pub(crate) width: Size,
    pub(crate) height: Size,

    pub(crate) margin: Margin,
    pub(crate) margin_left: Option<Margin>,
    pub(crate) margin_right: Option<Margin>,
    pub(crate) margin_top: Option<Margin>,
    pub(crate) margin_bottom: Option<Margin>,

    pub(crate) padding: Padding,
    pub(crate) padding_left: Option<Padding>,
    pub(crate) padding_right: Option<Padding>,
    pub(crate) padding_top: Option<Padding>,
    pub(crate) padding_bottom: Option<Padding>,

    pub(crate) gap: Gap,
    pub(crate) column_gap: Option<Gap>,
    pub(crate) row_gap: Option<Gap>,

    pub(crate) overflow: Option<Overflow>,

    pub(crate) flex_direction: Option<FlexDirection>,
    pub(crate) flex_wrap: Option<FlexWrap>,

    pub(crate) flex_grow: Option<f32>,
    pub(crate) flex_shrink: Option<f32>,

    pub(crate) flex_basis: Option<FlexBasis>,
    pub(crate) align_items: Option<AlignItems>,

    pub(crate) align_self: Option<AlignSelf>,
    pub(crate) align_content: Option<AlignContent>,

    pub(crate) justify_content: Option<JustifyContent>,

    // TODO: add css grid

    // TODO: split this up
    pub(crate) bstyle: BtnStyle,
}

// The final conversion from SendableStyle -> taffy::Style
impl From<&Style> for taffy::Style {
    fn from(val: &Style) -> Self {
        Self {
            display: val.display.into(),
            position: val.position.into(),
            inset: taffy::Rect {
                left: val.inset_left.unwrap_or(val.inset).into(),
                right: val.inset_right.unwrap_or(val.inset).into(),
                top: val.inset_top.unwrap_or(val.inset).into(),
                bottom: val.inset_bottom.unwrap_or(val.inset).into(),
            },
            size: taffy::Size {
                width: val.width.into(),
                height: val.height.into(),
            },
            margin: taffy::Rect {
                left: val.margin_left.unwrap_or(val.margin).into(),
                right: val.margin_right.unwrap_or(val.margin).into(),
                top: val.margin_top.unwrap_or(val.margin).into(),
                bottom: val.margin_top.unwrap_or(val.margin).into(),
            },
            padding: taffy::Rect {
                left: val.padding_left.unwrap_or(val.padding).into(),
                right: val.padding_right.unwrap_or(val.padding).into(),
                top: val.padding_top.unwrap_or(val.padding).into(),
                bottom: val.padding_bottom.unwrap_or(val.padding).into(),
            },
            gap: taffy::Size {
                width: val.column_gap.unwrap_or(val.gap).into(),
                height: val.row_gap.unwrap_or(val.gap).into(),
            },
            flex_direction: val.flex_direction.unwrap_or_default().into(),
            flex_wrap: val.flex_wrap.unwrap_or_default().into(),
            flex_grow: val.flex_grow.unwrap_or(0.0) as f32,
            flex_shrink: val.flex_shrink.unwrap_or(1.0) as f32,
            flex_basis: val.flex_basis.unwrap_or_default().into(),
            align_items: val.align_items.map(|v| v.into()),
            align_self: val.align_self.map(|v| v.into()),
            align_content: val.align_content.map(|v| v.into()),
            justify_content: val.justify_content.map(|v| v.into()),
            // Use Taffy's defaults for any fields not covered to ensure forward compatibility
            ..Self::DEFAULT
        }
    }
}

impl From<&Style> for BtnStyle {
    fn from(value: &Style) -> Self {
        value.bstyle.clone()
    }
}
