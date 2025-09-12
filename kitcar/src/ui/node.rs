//! UI Node and associated items
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
        style: SendableStyle,
        /// Button Text
        text: Cow<'static, str>,
        /// *Your* ClickId - the ClickId sent to LFS will be assigned
        key: UINodeKey,
    },
    /// Unrendered items are just used to help the layout generation
    Unrendered {
        /// Requested layout
        // XXX: see above
        style: SendableStyle,
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

    pub(crate) fn hash(&self, layout: &taffy::Layout) -> Option<u64> {
        if let UINode::Rendered {
            text, key, style, ..
        } = self
        {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            text.hash(&mut hasher);
            style.hash(&mut hasher);
            layout.location.x.to_bits().hash(&mut hasher);
            layout.location.y.to_bits().hash(&mut hasher);
            layout.size.width.to_bits().hash(&mut hasher);
            layout.size.height.to_bits().hash(&mut hasher);
            Some(hasher.finish())
        } else {
            None
        }
    }

    pub(crate) fn children(&self) -> Option<&[Self]> {
        match self {
            Self::Unrendered { children, .. } => Some(children),
            Self::Rendered { .. } => None,
        }
    }

    // ... Builder methods start here ...

    /// Creates a new rendered node.
    pub fn rendered(text: impl Into<Cow<'static, str>>, key: UINodeKey) -> Self {
        Self::Rendered {
            style: SendableStyle::default(),
            text: text.into(),
            key,
        }
    }

    /// Creates a new unrendered node.
    pub fn unrendered() -> Self {
        Self::Unrendered {
            style: SendableStyle::default(),
            children: Vec::new(),
        }
    }

    // Helper method to get mutable reference to layout.
    /// Gets a mutable reference to layout.
    fn get_layout_mut(&mut self) -> &mut SendableStyle {
        match self {
            UINode::Rendered { style: layout, .. } => layout,
            UINode::Unrendered { style: layout, .. } => layout,
        }
    }

    // Helper method to get immutable reference to layout.
    /// Gets an immutable reference to layout.
    pub fn get_layout(&self) -> &SendableStyle {
        match self {
            UINode::Rendered { style: layout, .. } => layout,
            UINode::Unrendered { style: layout, .. } => layout,
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

    // Display and positioning
    fn with_display(mut self, display: SendableDisplay) -> Self {
        self.get_layout_mut().display = display;
        self
    }

    // Sets the position property.
    fn with_position(mut self, position: SendablePosition) -> Self {
        self.get_layout_mut().position = position;
        self
    }

    /// Sets the display to Flex.
    pub fn display_flex(self) -> Self {
        self.with_display(SendableDisplay::Flex)
    }

    /// Sets the display to Grid.
    pub fn display_grid(self) -> Self {
        self.with_display(SendableDisplay::Grid)
    }

    /// Sets the display to Block.
    pub fn display_block(self) -> Self {
        self.with_display(SendableDisplay::Block)
    }

    /// Sets the display to None.
    pub fn display_none(self) -> Self {
        self.with_display(SendableDisplay::None)
    }

    /// Sets the position to Absolute.
    pub fn position_absolute(self) -> Self {
        self.with_position(SendablePosition::Absolute)
    }

    /// Sets the position to Relative.
    pub fn position_relative(self) -> Self {
        self.with_position(SendablePosition::Relative)
    }

    fn with_width(mut self, width: SendableDimension) -> Self {
        self.get_layout_mut().size.width = width;
        self
    }

    fn with_height(mut self, height: SendableDimension) -> Self {
        self.get_layout_mut().size.height = height;
        self
    }

    // Convenience size methods
    /// Sets the width in pixels.
    pub fn width(self, px: u8) -> Self {
        self.with_width(SendableDimension::Points(px))
    }

    /// Sets the height in pixels.
    pub fn height(self, px: u8) -> Self {
        self.with_height(SendableDimension::Points(px))
    }

    // Margin methods
    fn with_margin(mut self, margin: SendableRect<SendableLengthAuto>) -> Self {
        self.get_layout_mut().margin = margin;
        self
    }

    /// Sets the left margin.
    pub fn with_margin_left(mut self, left: SendableLengthAuto) -> Self {
        self.get_layout_mut().margin.left = left;
        self
    }

    /// Sets the right margin.
    pub fn with_margin_right(mut self, right: SendableLengthAuto) -> Self {
        self.get_layout_mut().margin.right = right;
        self
    }

    /// Sets the top margin.
    pub fn with_margin_top(mut self, top: SendableLengthAuto) -> Self {
        self.get_layout_mut().margin.top = top;
        self
    }

    /// Sets the bottom margin.
    pub fn with_margin_bottom(mut self, bottom: SendableLengthAuto) -> Self {
        self.get_layout_mut().margin.bottom = bottom;
        self
    }

    /// Sets all margins to the same value.
    pub fn margin(self, all: u8) -> Self {
        let margin = SendableLengthAuto::Points(all);
        self.with_margin(SendableRect {
            left: margin,
            right: margin,
            top: margin,
            bottom: margin,
        })
    }

    /// Sets all margins individually.
    pub fn margin_all(self, top: u8, right: u8, bottom: u8, left: u8) -> Self {
        self.with_margin(SendableRect {
            left: SendableLengthAuto::Points(left),
            right: SendableLengthAuto::Points(right),
            top: SendableLengthAuto::Points(top),
            bottom: SendableLengthAuto::Points(bottom),
        })
    }

    /// Sets horizontal margins.
    pub fn margin_x(self, horizontal: u8) -> Self {
        let margin = SendableLengthAuto::Points(horizontal);
        self.with_margin_left(margin).with_margin_right(margin)
    }

    /// Sets vertical margins.
    pub fn margin_y(self, vertical: u8) -> Self {
        let margin = SendableLengthAuto::Points(vertical);
        self.with_margin_top(margin).with_margin_bottom(margin)
    }

    /// Sets all margins to auto.
    pub fn margin_auto(self) -> Self {
        self.with_margin(SendableRect {
            left: SendableLengthAuto::Auto,
            right: SendableLengthAuto::Auto,
            top: SendableLengthAuto::Auto,
            bottom: SendableLengthAuto::Auto,
        })
    }

    // Padding methods
    fn with_padding(mut self, padding: SendableRect<SendableLength>) -> Self {
        self.get_layout_mut().padding = padding;
        self
    }

    /// Sets the left padding.
    pub fn with_padding_left(mut self, left: SendableLength) -> Self {
        self.get_layout_mut().padding.left = left;
        self
    }

    /// Sets the right padding.
    pub fn with_padding_right(mut self, right: SendableLength) -> Self {
        self.get_layout_mut().padding.right = right;
        self
    }

    /// Sets the top padding.
    pub fn with_padding_top(mut self, top: SendableLength) -> Self {
        self.get_layout_mut().padding.top = top;
        self
    }

    /// Sets the bottom padding.
    pub fn with_padding_bottom(mut self, bottom: SendableLength) -> Self {
        self.get_layout_mut().padding.bottom = bottom;
        self
    }

    /// Sets all padding to the same value.
    pub fn padding(self, all: u8) -> Self {
        let padding = SendableLength { points: all };
        self.with_padding(SendableRect {
            left: padding,
            right: padding,
            top: padding,
            bottom: padding,
        })
    }

    /// Sets all padding individually.
    pub fn padding_all(self, top: u8, right: u8, bottom: u8, left: u8) -> Self {
        self.with_padding(SendableRect {
            left: SendableLength { points: left },
            right: SendableLength { points: right },
            top: SendableLength { points: top },
            bottom: SendableLength { points: bottom },
        })
    }

    /// Sets horizontal padding.
    pub fn padding_x(self, horizontal: u8) -> Self {
        let padding = SendableLength { points: horizontal };
        self.with_padding_left(padding).with_padding_right(padding)
    }

    /// Sets vertical padding.
    pub fn padding_y(self, vertical: u8) -> Self {
        let padding = SendableLength { points: vertical };
        self.with_padding_top(padding).with_padding_bottom(padding)
    }

    // Inset methods (for absolute positioning)
    /// Sets the inset property.
    pub fn with_inset(mut self, inset: SendableRect<SendableLengthAuto>) -> Self {
        self.get_layout_mut().inset = inset;
        self
    }

    /// Sets the left inset.
    pub fn with_left(mut self, left: SendableLengthAuto) -> Self {
        self.get_layout_mut().inset.left = left;
        self
    }

    /// Sets the right inset.
    pub fn with_right(mut self, right: SendableLengthAuto) -> Self {
        self.get_layout_mut().inset.right = right;
        self
    }

    /// Sets the top inset.
    pub fn with_top(mut self, top: SendableLengthAuto) -> Self {
        self.get_layout_mut().inset.top = top;
        self
    }

    /// Sets the bottom inset.
    pub fn with_bottom(mut self, bottom: SendableLengthAuto) -> Self {
        self.get_layout_mut().inset.bottom = bottom;
        self
    }

    /// Sets the left inset in pixels.
    pub fn left(self, px: u8) -> Self {
        self.with_left(SendableLengthAuto::Points(px))
    }

    /// Sets the right inset in pixels.
    pub fn right(self, px: u8) -> Self {
        self.with_right(SendableLengthAuto::Points(px))
    }

    /// Sets the top inset in pixels.
    pub fn top(self, px: u8) -> Self {
        self.with_top(SendableLengthAuto::Points(px))
    }

    /// Sets the bottom inset in pixels.
    pub fn bottom(self, px: u8) -> Self {
        self.with_bottom(SendableLengthAuto::Points(px))
    }

    // Flexbox methods
    /// Sets the flex direction.
    pub fn with_flex_direction(mut self, direction: SendableFlexDirection) -> Self {
        self.get_layout_mut().flex_direction = direction;
        self
    }

    /// Sets the flex wrap.
    pub fn with_flex_wrap(mut self, wrap: SendableFlexWrap) -> Self {
        self.get_layout_mut().flex_wrap = wrap;
        self
    }

    /// Sets the flex grow factor.
    pub fn with_flex_grow(mut self, grow: u8) -> Self {
        self.get_layout_mut().flex_grow = grow;
        self
    }

    /// Sets the flex shrink factor.
    pub fn with_flex_shrink(mut self, shrink: u8) -> Self {
        self.get_layout_mut().flex_shrink = shrink;
        self
    }

    /// Sets the flex basis.
    pub fn with_flex_basis(mut self, basis: SendableDimension) -> Self {
        self.get_layout_mut().flex_basis = basis;
        self
    }

    /// Sets flex and direction to row.
    pub fn flex_direction_row(self) -> Self {
        self.display_flex()
            .with_flex_direction(SendableFlexDirection::Row)
    }

    /// Sets flex and direction to column.
    pub fn flex_direction_column(self) -> Self {
        self.display_flex()
            .with_flex_direction(SendableFlexDirection::Column)
    }

    /// Sets flex and direction to row_reverse.
    pub fn flex_direction_row_reverse(self) -> Self {
        self.display_flex()
            .with_flex_direction(SendableFlexDirection::RowReverse)
    }

    /// Sets flex and direction to column_reverse.
    pub fn flex_direction_column_reverse(self) -> Self {
        self.display_flex()
            .with_flex_direction(SendableFlexDirection::ColumnReverse)
    }

    /// Sets flex wrap to wrap.
    pub fn flex_wrap(self) -> Self {
        self.with_flex_wrap(SendableFlexWrap::Wrap)
    }

    /// Sets flex wrap to nowrap.
    pub fn flex_nowrap(self) -> Self {
        self.with_flex_wrap(SendableFlexWrap::NoWrap)
    }

    /// Sets flex wrap to wrap_reverse.
    pub fn flex_wrap_reverse(self) -> Self {
        self.with_flex_wrap(SendableFlexWrap::WrapReverse)
    }

    /// Sets the flex grow factor.
    pub fn flex_grow(self, factor: u8) -> Self {
        self.with_flex_grow(factor)
    }

    /// Sets the flex shrink factor.
    pub fn flex_shrink(self, factor: u8) -> Self {
        self.with_flex_shrink(factor)
    }

    /// Sets flex basis in pixels.
    pub fn flex_basis(self, px: u8) -> Self {
        self.with_flex_basis(SendableDimension::Points(px))
    }

    /// Sets flex basis to auto.
    pub fn flex_basis_auto(self) -> Self {
        self.with_flex_basis(SendableDimension::Auto)
    }

    // Alignment methods
    fn with_align_items(mut self, align: SendableAlignItems) -> Self {
        self.get_layout_mut().align_items = Some(align);
        self
    }

    fn with_justify_content(mut self, justify: SendableJustifyContent) -> Self {
        self.get_layout_mut().justify_content = Some(justify);
        self
    }

    // Convenience alignment methods
    /// Centers children vertically and horizontally.
    pub fn center(self) -> Self {
        self.with_align_items(SendableAlignItems::Center)
            .with_justify_content(SendableJustifyContent::Center)
    }

    /// Centers children vertically.
    pub fn align_items_center(self) -> Self {
        self.with_align_items(SendableAlignItems::Center)
    }

    /// Centers children horizontally.
    pub fn justify_content_center(self) -> Self {
        self.with_justify_content(SendableJustifyContent::Center)
    }

    /// Aligns items to the start.
    pub fn start_items(self) -> Self {
        self.with_align_items(SendableAlignItems::Start)
    }

    /// Aligns items to the end.
    pub fn end_items(self) -> Self {
        self.with_align_items(SendableAlignItems::End)
    }

    /// Stretches items to fill container.
    pub fn align_items_stretch(self) -> Self {
        self.with_align_items(SendableAlignItems::Stretch)
    }

    /// Flex start
    pub fn align_items_flex_start(self) -> Self {
        self.with_align_items(SendableAlignItems::FlexStart)
    }

    /// Flex end
    pub fn align_items_flex_end(self) -> Self {
        self.with_align_items(SendableAlignItems::FlexEnd)
    }

    /// Justifies content to the start.
    pub fn justify_content_start(self) -> Self {
        self.with_justify_content(SendableJustifyContent::Start)
    }

    /// Justifies content to the end.
    pub fn justify_content_end(self) -> Self {
        self.with_justify_content(SendableJustifyContent::End)
    }

    /// Spaces content evenly.
    pub fn justify_content_space_between(self) -> Self {
        self.with_justify_content(SendableJustifyContent::SpaceBetween)
    }

    /// Spaces content with space around.
    pub fn justify_content_space_around(self) -> Self {
        self.with_justify_content(SendableJustifyContent::SpaceAround)
    }

    /// Spaces content with space evenly.
    pub fn justify_content_space_evenly(self) -> Self {
        self.with_justify_content(SendableJustifyContent::SpaceEvenly)
    }

    /// Justify content flex start
    pub fn justify_content_flex_start(self) -> Self {
        self.with_justify_content(SendableJustifyContent::FlexStart)
    }

    /// Justify content flex end
    pub fn justify_content_flex_end(self) -> Self {
        self.with_justify_content(SendableJustifyContent::FlexEnd)
    }

    // Gap methods
    fn with_gap(mut self, gap: SendableSize<SendableLength>) -> Self {
        self.get_layout_mut().gap = gap;
        self
    }

    fn with_row_gap(mut self, gap: SendableLength) -> Self {
        self.get_layout_mut().gap.height = gap;
        self
    }

    fn with_column_gap(mut self, gap: SendableLength) -> Self {
        self.get_layout_mut().gap.width = gap;
        self
    }

    /// Sets both row and column gap.
    pub fn gap(self, gap: u8) -> Self {
        let gap_val = SendableLength { points: gap };
        self.with_gap(SendableSize {
            width: gap_val,
            height: gap_val,
        })
    }

    /// Sets horizontal and vertical gaps.
    pub fn gap_xy(self, x: u8, y: u8) -> Self {
        self.with_gap(SendableSize {
            width: SendableLength { points: x },
            height: SendableLength { points: y },
        })
    }

    /// Sets the row gap.
    pub fn row_gap(self, gap: u8) -> Self {
        self.with_row_gap(SendableLength { points: gap })
    }

    /// Sets the column gap.
    pub fn column_gap(self, gap: u8) -> Self {
        self.with_column_gap(SendableLength { points: gap })
    }

    // LFS button styles

    /// Light grey / NotEditable
    pub fn light_grey(mut self) -> Self {
        self.get_layout_mut().bstyle.colour = BtnStyleColour::NotEditable;
        self
    }

    /// Yellow/ Title
    pub fn yellow(mut self) -> Self {
        self.get_layout_mut().bstyle.colour = BtnStyleColour::Title;
        self
    }

    /// Black / UnselectedText
    pub fn black(mut self) -> Self {
        self.get_layout_mut().bstyle.colour = BtnStyleColour::UnselectedText;
        self
    }

    /// White / SelectedText
    pub fn white(mut self) -> Self {
        self.get_layout_mut().bstyle.colour = BtnStyleColour::SelectedText;
        self
    }

    /// Green / Ok
    pub fn green(mut self) -> Self {
        self.get_layout_mut().bstyle.colour = BtnStyleColour::Ok;
        self
    }

    /// Red / Cancel
    pub fn red(mut self) -> Self {
        self.get_layout_mut().bstyle.colour = BtnStyleColour::Cancel;
        self
    }

    /// Pale blue / TextString
    pub fn pale_blue(mut self) -> Self {
        self.get_layout_mut().bstyle.colour = BtnStyleColour::TextString;
        self
    }

    /// Grey / Unavailable
    pub fn grey(mut self) -> Self {
        self.get_layout_mut().bstyle.colour = BtnStyleColour::Unavailable;
        self
    }

    /// Set button as clickable
    pub fn clickable(mut self) -> Self {
        self.get_layout_mut()
            .bstyle
            .flags
            .set(BtnStyleFlags::CLICK, true);
        self
    }

    /// Light button
    pub fn light(mut self) -> Self {
        self.get_layout_mut()
            .bstyle
            .flags
            .set(BtnStyleFlags::LIGHT, true);
        self.get_layout_mut()
            .bstyle
            .flags
            .set(BtnStyleFlags::DARK, false);
        self
    }

    /// Dark button
    pub fn dark(mut self) -> Self {
        self.get_layout_mut()
            .bstyle
            .flags
            .set(BtnStyleFlags::LIGHT, false);
        self.get_layout_mut()
            .bstyle
            .flags
            .set(BtnStyleFlags::DARK, true);
        self
    }

    /// Align text left
    pub fn align_left(mut self) -> Self {
        self.get_layout_mut()
            .bstyle
            .flags
            .set(BtnStyleFlags::LEFT, true);
        self
    }

    /// Align text right
    pub fn align_right(mut self) -> Self {
        self.get_layout_mut()
            .bstyle
            .flags
            .set(BtnStyleFlags::RIGHT, true);
        self
    }
}
