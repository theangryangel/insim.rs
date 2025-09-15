//! A Send + Sync capable reimplementation of the parts of `taffy::Style` used in the project.

use std::{fmt::Debug, hash::Hash};

use insim::insim::BtnStyle;
use taffy::Rect;

/// A `Send`-capable equivalent of `taffy::geometry::Size`.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub(crate) struct SendableSize<T: Hash + Debug> {
    pub(crate) width: T,
    pub(crate) height: T,
}

impl<T: Hash + Debug + Default> Default for SendableSize<T> {
    fn default() -> Self {
        Self {
            width: T::default(),
            height: T::default(),
        }
    }
}

/// A `Send`-capable equivalent of `taffy::geometry::Rect`.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub(crate) struct SendableRect<T> {
    pub(crate) left: T,
    pub(crate) top: T,
    pub(crate) right: T,
    pub(crate) bottom: T,
}

impl<T: Default> Default for SendableRect<T> {
    fn default() -> Self {
        Self {
            left: T::default(),
            top: T::default(),
            right: T::default(),
            bottom: T::default(),
        }
    }
}

//--------------------------------------------------------------------------------//
//                       VALUE ENUMS (NO PERCENTAGES)                             //
//--------------------------------------------------------------------------------//

/// A `Send`-capable equivalent of `taffy::style::Dimension`.
/// Does not include a percentage variant.
#[derive(Copy, Clone, Debug, PartialEq, Default, Hash)]
pub(crate) enum SendableDimension {
    #[default]
    Auto,
    Points(u8),
}

/// A `Send`-capable equivalent of `taffy::style::LengthPercentageAuto`.
/// Does not include a percentage variant.
#[derive(Copy, Clone, Debug, PartialEq, Default, Hash)]
pub(crate) enum SendableLengthAuto {
    Points(u8),
    #[default]
    Auto,
}

/// A `Send`-capable equivalent of `taffy::style::LengthPercentage`.
/// Does not include a percentage variant.
#[derive(Copy, Clone, Debug, PartialEq, Default, Hash)]
pub(crate) struct SendableLength {
    pub points: u8,
}

//--------------------------------------------------------------------------------//
//                            LAYOUT ENUMS                                        //
//--------------------------------------------------------------------------------//

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Hash)]
pub(crate) enum SendableDisplay {
    #[default]
    Flex,
    Grid,
    Block,
    None,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Hash)]
pub(crate) enum SendablePosition {
    #[default]
    Relative,
    Absolute,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Hash)]
pub(crate) enum SendableFlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Hash)]
pub(crate) enum SendableFlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum SendableAlignItems {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum SendableAlignSelf {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum SendableAlignContent {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum SendableJustifyContent {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// A `Send`-capable replacement for `taffy::style::Style`.
///
/// This struct can be safely sent across threads. It implements `From<SendableStyle>`
/// for `taffy::style::Style` to allow for easy conversion before passing it to Taffy.
// FIXME: This is all fucked and doesnt match the Taffy defaults, resulting me wasting my time.
// Considering solutions.
// This is great because it doesnt expose taffy to the user.
// It's shit because we need to keep in sync with upstream defaults.
// This is great because its Send and taffy 0.8 aint.
// Maybe we just downgrade to 0.7 for now?
#[derive(Clone, Debug, Hash)]
pub struct SendableStyle {
    pub(crate) display: SendableDisplay,
    pub(crate) position: SendablePosition,
    pub(crate) inset: SendableRect<SendableLengthAuto>,
    pub(crate) size: SendableSize<SendableDimension>,
    pub(crate) margin: SendableRect<SendableLengthAuto>,
    pub(crate) padding: SendableRect<SendableLength>,
    pub(crate) gap: SendableSize<SendableLength>,
    pub(crate) flex_direction: SendableFlexDirection,
    pub(crate) flex_wrap: SendableFlexWrap,
    pub(crate) flex_grow: u8,
    pub(crate) flex_shrink: u8,
    pub(crate) flex_basis: SendableDimension,
    pub(crate) align_items: Option<SendableAlignItems>,
    pub(crate) align_self: Option<SendableAlignSelf>,
    pub(crate) align_content: Option<SendableAlignContent>,
    pub(crate) justify_content: Option<SendableJustifyContent>,

    pub(crate) bstyle: BtnStyle,
}

impl Default for SendableStyle {
    /// Creates a new `SendableStyle` with values that match `taffy::style::Style::DEFAULT`.
    fn default() -> Self {
        Self {
            display: SendableDisplay::default(),
            position: SendablePosition::default(),
            inset: SendableRect::default(),
            size: SendableSize::default(),
            // FIXME margin and padding are defaulting to auto and its all going to tits.
            // Should we just downgrade taffy :/ because lazy
            margin: SendableRect::default(),
            padding: SendableRect::default(),
            gap: SendableSize::default(),
            flex_direction: SendableFlexDirection::default(),
            flex_wrap: SendableFlexWrap::default(),
            flex_grow: 0,
            flex_shrink: 1,
            flex_basis: SendableDimension::default(),
            align_items: None,
            align_self: None,
            align_content: None,
            justify_content: None,

            bstyle: BtnStyle::default(),
        }
    }
}

// From implementations for value types
impl From<SendableDimension> for taffy::Dimension {
    fn from(val: SendableDimension) -> Self {
        match val {
            SendableDimension::Auto => Self::auto(),
            SendableDimension::Points(p) => Self::length(p as f32),
        }
    }
}

impl From<SendableLengthAuto> for taffy::LengthPercentageAuto {
    fn from(val: SendableLengthAuto) -> Self {
        match val {
            SendableLengthAuto::Auto => Self::auto(),
            SendableLengthAuto::Points(p) => Self::length(p as f32),
        }
    }
}

impl From<SendableLength> for taffy::LengthPercentage {
    fn from(val: SendableLength) -> Self {
        Self::length(val.points as f32)
    }
}

// From implementations for enums
impl From<SendableDisplay> for taffy::Display {
    fn from(val: SendableDisplay) -> Self {
        match val {
            SendableDisplay::Flex => Self::Flex,
            SendableDisplay::Grid => Self::Grid,
            SendableDisplay::Block => Self::Block,
            SendableDisplay::None => Self::None,
        }
    }
}

impl From<SendablePosition> for taffy::Position {
    fn from(val: SendablePosition) -> Self {
        match val {
            SendablePosition::Relative => Self::Relative,
            SendablePosition::Absolute => Self::Absolute,
        }
    }
}

impl From<SendableFlexDirection> for taffy::FlexDirection {
    fn from(val: SendableFlexDirection) -> Self {
        match val {
            SendableFlexDirection::Row => Self::Row,
            SendableFlexDirection::Column => Self::Column,
            SendableFlexDirection::RowReverse => Self::RowReverse,
            SendableFlexDirection::ColumnReverse => Self::ColumnReverse,
        }
    }
}

impl From<SendableFlexWrap> for taffy::FlexWrap {
    fn from(val: SendableFlexWrap) -> Self {
        match val {
            SendableFlexWrap::NoWrap => Self::NoWrap,
            SendableFlexWrap::Wrap => Self::Wrap,
            SendableFlexWrap::WrapReverse => Self::WrapReverse,
        }
    }
}

impl From<SendableAlignItems> for taffy::AlignItems {
    fn from(val: SendableAlignItems) -> Self {
        match val {
            SendableAlignItems::Start => Self::Start,
            SendableAlignItems::End => Self::End,
            SendableAlignItems::FlexStart => Self::FlexStart,
            SendableAlignItems::FlexEnd => Self::FlexEnd,
            SendableAlignItems::Center => Self::Center,
            SendableAlignItems::Baseline => Self::Baseline,
            SendableAlignItems::Stretch => Self::Stretch,
        }
    }
}

impl From<SendableAlignSelf> for taffy::AlignSelf {
    fn from(val: SendableAlignSelf) -> Self {
        match val {
            SendableAlignSelf::Start => Self::Start,
            SendableAlignSelf::End => Self::End,
            SendableAlignSelf::FlexStart => Self::FlexStart,
            SendableAlignSelf::FlexEnd => Self::FlexEnd,
            SendableAlignSelf::Center => Self::Center,
            SendableAlignSelf::Baseline => Self::Baseline,
            SendableAlignSelf::Stretch => Self::Stretch,
        }
    }
}

impl From<SendableAlignContent> for taffy::AlignContent {
    fn from(val: SendableAlignContent) -> Self {
        match val {
            SendableAlignContent::Start => Self::Start,
            SendableAlignContent::End => Self::End,
            SendableAlignContent::FlexStart => Self::FlexStart,
            SendableAlignContent::FlexEnd => Self::FlexEnd,
            SendableAlignContent::Center => Self::Center,
            SendableAlignContent::Stretch => Self::Stretch,
            SendableAlignContent::SpaceBetween => Self::SpaceBetween,
            SendableAlignContent::SpaceAround => Self::SpaceAround,
            SendableAlignContent::SpaceEvenly => Self::SpaceEvenly,
        }
    }
}

impl From<SendableJustifyContent> for taffy::JustifyContent {
    fn from(val: SendableJustifyContent) -> Self {
        match val {
            SendableJustifyContent::Start => Self::Start,
            SendableJustifyContent::End => Self::End,
            SendableJustifyContent::FlexStart => Self::FlexStart,
            SendableJustifyContent::FlexEnd => Self::FlexEnd,
            SendableJustifyContent::Center => Self::Center,
            SendableJustifyContent::SpaceBetween => Self::SpaceBetween,
            SendableJustifyContent::SpaceAround => Self::SpaceAround,
            SendableJustifyContent::SpaceEvenly => Self::SpaceEvenly,
        }
    }
}

// From implementations for generic structs
impl<T, U> From<SendableRect<T>> for taffy::Rect<U>
where
    T: Into<U> + Copy,
{
    fn from(val: SendableRect<T>) -> Self {
        Self {
            left: val.left.into(),
            right: val.right.into(),
            top: val.top.into(),
            bottom: val.bottom.into(),
        }
    }
}

impl<T, U> From<SendableSize<T>> for taffy::Size<U>
where
    T: Into<U> + Debug + Hash + Copy,
{
    fn from(val: SendableSize<T>) -> Self {
        Self {
            width: val.width.into(),
            height: val.height.into(),
        }
    }
}

// The final conversion from SendableStyle -> taffy::Style
impl From<&SendableStyle> for taffy::Style {
    fn from(val: &SendableStyle) -> Self {
        Self {
            display: val.display.into(),
            position: val.position.into(),
            size: val.size.into(),
            // margin: val.margin.into(),
            padding: val.padding.into(),
            gap: val.gap.into(),
            flex_direction: val.flex_direction.into(),
            flex_wrap: val.flex_wrap.into(),
            flex_grow: val.flex_grow as f32,
            flex_shrink: val.flex_shrink as f32,
            flex_basis: val.flex_basis.into(),
            align_items: val.align_items.map(|v| v.into()),
            align_self: val.align_self.map(|v| v.into()),
            align_content: val.align_content.map(|v| v.into()),
            justify_content: val.justify_content.map(|v| v.into()),
            // Use Taffy's defaults for any fields not covered to ensure forward compatibility
            ..Self::DEFAULT
        }
    }
}

impl From<&SendableStyle> for BtnStyle {
    fn from(value: &SendableStyle) -> Self {
        value.bstyle.clone()
    }
}
