//! UI Node and associated items
use std::{
    borrow::Cow,
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
    ops::{Deref, DerefMut},
};

use insim::insim::BtnStyle;
use taffy::{prelude::length, AlignContent, AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap, JustifyContent, Layout, LengthPercentage, LengthPercentageAuto, Position, Rect, Size, Style};


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
        layout: Style,
        /// Button style
        style: BtnStyle,
        /// Button Text
        text: Cow<'static, str>,
        /// *Your* ClickId - the ClickId sent to LFS will be assigned
        key: UINodeKey,
        /// Child nodes
        children: Vec<Self>,
    },
    /// Unrendered items are just used to help the layout generation
    Unrendered {
        /// Requested layout
        layout: Style,
        /// Child nodes
        children: Vec<Self>,
    },
}

impl UINode {
    pub(crate) fn bstyle(&self) -> Option<BtnStyle> {
        match self {
            UINode::Rendered { style, .. } => Some(style.clone()),
            UINode::Unrendered { .. } => None,
        }
    }

    pub(crate) fn text(&self) -> Option<Cow<'static, str>> {
        match self {
            UINode::Rendered { text, .. } => Some(text.clone()),
            UINode::Unrendered { .. } => None,
        }
    }

    pub(crate) fn hash(&self, layout: &Layout) -> Option<u64> {
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

    pub(crate) fn children(&self) -> &[Self] {
        match self {
            Self::Unrendered { children, .. } => children,
            Self::Rendered { children, .. } => children,
        }
    }

    // ... Builder methods start here ...

    /// Creates a new rendered node.
    pub fn rendered(
        style: BtnStyle, 
        text: impl Into<Cow<'static, str>>, 
        key: UINodeKey
    ) -> Self {
        Self::Rendered {
            layout: Style::default(),
            style,
            text: text.into(),
            key,
            children: Vec::new(),
        }
    }

    /// Creates a new unrendered node.
    pub fn unrendered() -> Self {
        Self::Unrendered {
            layout: Style::default(),
            children: Vec::new(),
        }
    }

    // Helper method to get mutable reference to layout
    /// Gets a mutable reference to layout.
    fn get_layout_mut(&mut self) -> &mut Style {
        match self {
            UINode::Rendered { layout, .. } => layout,
            UINode::Unrendered { layout, .. } => layout,
        }
    }

    // Helper method to get immutable reference to layout
    /// Gets an immutable reference to layout.
    pub fn get_layout(&self) -> &Style {
        match self {
            UINode::Rendered { layout, .. } => layout,
            UINode::Unrendered { layout, .. } => layout,
        }
    }

    // Child management
    /// Adds a single child.
    pub fn with_child(mut self, child: UINode) -> Self {
        match &mut self {
            UINode::Rendered { children, .. } => children.push(child),
            UINode::Unrendered { children, .. } => children.push(child),
        }
        self
    }


    // pub fn release<'a, I>(&mut self, click_ids: &'a I)
    // where
    //     &'a I: IntoIterator<Item = &'a ClickId>,
    // {

    /// Adds a list of children.
    pub fn with_children<I>(mut self, new_children: I) -> Self
    where 
        I: IntoIterator<Item = Self> {
        match &mut self {
            UINode::Rendered { children, .. } => children.extend(new_children),
            UINode::Unrendered { children, .. } => children.extend(new_children),
        }
        self
    }

    // Display and positioning
    fn with_display(mut self, display: Display) -> Self {
        self.get_layout_mut().display = display;
        self
    }

    // Sets the position property.
    fn with_position(mut self, position: Position) -> Self {
        self.get_layout_mut().position = position;
        self
    }

    /// Sets the display to Flex.
    pub fn display_flex(self) -> Self {
        self.with_display(Display::Flex)
    }

    /// Sets the display to Grid.
    pub fn display_grid(self) -> Self {
        self.with_display(Display::Grid)
    }

    /// Sets the display to Block.
    pub fn display_block(self) -> Self {
        self.with_display(Display::Block)
    }

    /// Sets the display to None.
    pub fn display_none(self) -> Self {
        self.with_display(Display::None)
    }

    /// Sets the position to Absolute.
    pub fn position_absolute(self) -> Self {
        self.with_position(Position::Absolute)
    }

    /// Sets the position to Relative.
    pub fn position_relative(self) -> Self {
        self.with_position(Position::Relative)
    }

    fn with_width(mut self, width: Dimension) -> Self {
        self.get_layout_mut().size.width = width;
        self
    }

    fn with_height(mut self, height: Dimension) -> Self {
        self.get_layout_mut().size.height = height;
        self
    }

    // Convenience size methods
    /// Sets the width in pixels.
    pub fn width(self, px: f32) -> Self {
        self.with_width(length(px))
    }

    /// Sets the height in pixels.
    pub fn height(self, px: f32) -> Self {
        self.with_height(length(px))
    }

    /// Sets the width as a percentage.
    pub fn width_percent(self, percent: f32) -> Self {
        self.with_width(Dimension::percent(percent))
    }

    /// Sets the height as a percentage.
    pub fn height_percent(self, percent: f32) -> Self {
        self.with_height(Dimension::percent(percent))
    }

    // Margin methods
    fn with_margin(mut self, margin: Rect<LengthPercentageAuto>) -> Self {
        self.get_layout_mut().margin = margin;
        self
    }

    /// Sets the left margin.
    pub fn with_margin_left(mut self, left: LengthPercentageAuto) -> Self {
        self.get_layout_mut().margin.left = left;
        self
    }

    /// Sets the right margin.
    pub fn with_margin_right(mut self, right: LengthPercentageAuto) -> Self {
        self.get_layout_mut().margin.right = right;
        self
    }

    /// Sets the top margin.
    pub fn with_margin_top(mut self, top: LengthPercentageAuto) -> Self {
        self.get_layout_mut().margin.top = top;
        self
    }

    /// Sets the bottom margin.
    pub fn with_margin_bottom(mut self, bottom: LengthPercentageAuto) -> Self {
        self.get_layout_mut().margin.bottom = bottom;
        self
    }

    /// Sets all margins to the same value.
    pub fn margin(self, all: f32) -> Self {
        self.with_margin(Rect::length(all))
    }

    /// Sets all margins individually.
    pub fn margin_all(self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.with_margin(Rect {
            left: LengthPercentageAuto::length(left),
            right: LengthPercentageAuto::length(right),
            top: LengthPercentageAuto::length(top),
            bottom: LengthPercentageAuto::length(bottom),
        })
    }

    /// Sets horizontal margins.
    pub fn margin_x(self, horizontal: f32) -> Self {
        let margin = LengthPercentageAuto::length(horizontal);
        self.with_margin_left(margin).with_margin_right(margin)
    }

    /// Sets vertical margins.
    pub fn margin_y(self, vertical: f32) -> Self {
        let margin = LengthPercentageAuto::length(vertical);
        self.with_margin_top(margin).with_margin_bottom(margin)
    }

    /// Sets all margins to auto.
    pub fn margin_auto(self) -> Self {
        self.with_margin(Rect::auto())
    }

    // Padding methods
    fn with_padding(mut self, padding: Rect<LengthPercentage>) -> Self {
        self.get_layout_mut().padding = padding;
        self
    }

    /// Sets the left padding.
    pub fn with_padding_left(mut self, left: LengthPercentage) -> Self {
        self.get_layout_mut().padding.left = left;
        self
    }

    /// Sets the right padding.
    pub fn with_padding_right(mut self, right: LengthPercentage) -> Self {
        self.get_layout_mut().padding.right = right;
        self
    }

    /// Sets the top padding.
    pub fn with_padding_top(mut self, top: LengthPercentage) -> Self {
        self.get_layout_mut().padding.top = top;
        self
    }

    /// Sets the bottom padding.
    pub fn with_padding_bottom(mut self, bottom: LengthPercentage) -> Self {
        self.get_layout_mut().padding.bottom = bottom;
        self
    }

    /// Sets all padding to the same value.
    pub fn padding(self, all: f32) -> Self {
        self.with_padding(Rect::length(all))
    }

    /// Sets all padding individually.
    pub fn padding_all(self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.with_padding(Rect {
            left: LengthPercentage::length(left),
            right: LengthPercentage::length(right),
            top: LengthPercentage::length(top),
            bottom: LengthPercentage::length(bottom),
        })
    }

    /// Sets horizontal padding.
    pub fn padding_x(self, horizontal: f32) -> Self {
        let padding = LengthPercentage::length(horizontal);
        self.with_padding_left(padding).with_padding_right(padding)
    }

    /// Sets vertical padding.
    pub fn padding_y(self, vertical: f32) -> Self {
        let padding = LengthPercentage::length(vertical);
        self.with_padding_top(padding).with_padding_bottom(padding)
    }

    // Inset methods (for absolute positioning)
    /// Sets the inset property.
    pub fn with_inset(mut self, inset: Rect<LengthPercentageAuto>) -> Self {
        self.get_layout_mut().inset = inset;
        self
    }

    /// Sets the left inset.
    pub fn with_left(mut self, left: LengthPercentageAuto) -> Self {
        self.get_layout_mut().inset.left = left;
        self
    }

    /// Sets the right inset.
    pub fn with_right(mut self, right: LengthPercentageAuto) -> Self {
        self.get_layout_mut().inset.right = right;
        self
    }

    /// Sets the top inset.
    pub fn with_top(mut self, top: LengthPercentageAuto) -> Self {
        self.get_layout_mut().inset.top = top;
        self
    }

    /// Sets the bottom inset.
    pub fn with_bottom(mut self, bottom: LengthPercentageAuto) -> Self {
        self.get_layout_mut().inset.bottom = bottom;
        self
    }

    /// Sets the left inset in pixels.
    pub fn left(self, px: f32) -> Self {
        self.with_left(LengthPercentageAuto::length(px))
    }

    /// Sets the right inset in pixels.
    pub fn right(self, px: f32) -> Self {
        self.with_right(LengthPercentageAuto::length(px))
    }

    /// Sets the top inset in pixels.
    pub fn top(self, px: f32) -> Self {
        self.with_top(LengthPercentageAuto::length(px))
    }

    /// Sets the bottom inset in pixels.
    pub fn bottom(self, px: f32) -> Self {
        self.with_bottom(LengthPercentageAuto::length(px))
    }

    // Flexbox methods
    /// Sets the flex direction.
    pub fn with_flex_direction(mut self, direction: FlexDirection) -> Self {
        self.get_layout_mut().flex_direction = direction;
        self
    }

    /// Sets the flex wrap.
    pub fn with_flex_wrap(mut self, wrap: FlexWrap) -> Self {
        self.get_layout_mut().flex_wrap = wrap;
        self
    }

    /// Sets the flex grow factor.
    pub fn with_flex_grow(mut self, grow: f32) -> Self {
        self.get_layout_mut().flex_grow = grow;
        self
    }

    /// Sets the flex shrink factor.
    pub fn with_flex_shrink(mut self, shrink: f32) -> Self {
        self.get_layout_mut().flex_shrink = shrink;
        self
    }

    /// Sets the flex basis.
    pub fn with_flex_basis(mut self, basis: Dimension) -> Self {
        self.get_layout_mut().flex_basis = basis;
        self
    }

    /// Sets flex and direction to row.
    pub fn flex_direction_row(self) -> Self {
        self.display_flex().with_flex_direction(FlexDirection::Row)
    }

    /// Sets flex and direction to column.
    pub fn flex_direction_column(self) -> Self {
        self.display_flex().with_flex_direction(FlexDirection::Column)
    }

    /// Sets flex and direction to row_reverse.
    pub fn flex_direction_row_reverse(self) -> Self {
        self.display_flex().with_flex_direction(FlexDirection::RowReverse)
    }

    /// Sets flex and direction to column_reverse.
    pub fn flex_direction_column_reverse(self) -> Self {
        self.display_flex().with_flex_direction(FlexDirection::ColumnReverse)
    }

    /// Sets flex wrap to wrap.
    pub fn flex_wrap(self) -> Self {
        self.with_flex_wrap(FlexWrap::Wrap)
    }

    /// Sets flex wrap to nowrap.
    pub fn flex_nowrap(self) -> Self {
        self.with_flex_wrap(FlexWrap::NoWrap)
    }

    /// Sets flex wrap to wrap_reverse.
    pub fn flex_wrap_reverse(self) -> Self {
        self.with_flex_wrap(FlexWrap::WrapReverse)
    }

    /// Sets the flex grow factor.
    pub fn flex_grow(self, factor: f32) -> Self {
        self.with_flex_grow(factor)
    }

    /// Sets the flex shrink factor.
    pub fn flex_shrink(self, factor: f32) -> Self {
        self.with_flex_shrink(factor)
    }

    /// Sets flex basis in pixels.
    pub fn flex_basis(self, px: f32) -> Self {
        self.with_flex_basis(Dimension::length(px))
    }

    /// Sets flex basis as a percentage.
    pub fn flex_basis_percent(self, percent: f32) -> Self {
        self.with_flex_basis(Dimension::percent(percent))
    }

    /// Sets flex basis to auto.
    pub fn flex_basis_auto(self) -> Self {
        self.with_flex_basis(Dimension::auto())
    }

    // Alignment methods
    fn with_align_items(mut self, align: AlignItems) -> Self {
        self.get_layout_mut().align_items = Some(align);
        self
    }

    fn with_align_self(mut self, align: AlignSelf) -> Self {
        self.get_layout_mut().align_self = Some(align);
        self
    }

    fn with_align_content(mut self, align: AlignContent) -> Self {
        self.get_layout_mut().align_content = Some(align);
        self
    }

    fn with_justify_content(mut self, justify: JustifyContent) -> Self {
        self.get_layout_mut().justify_content = Some(justify);
        self
    }

    // Convenience alignment methods
    /// Centers children vertically and horizontally.
    pub fn center(self) -> Self {
        self.with_align_items(AlignItems::Center)
            .with_justify_content(JustifyContent::Center)
    }

    /// Centers children vertically.
    pub fn align_items_center(self) -> Self {
        self.with_align_items(AlignItems::Center)
    }

    /// Centers children horizontally.
    pub fn justify_content_center(self) -> Self {
        self.with_justify_content(JustifyContent::Center)
    }

    /// Aligns items to the start.
    pub fn start_items(self) -> Self {
        self.with_align_items(AlignItems::Start)
    }

    /// Aligns items to the end.
    pub fn end_items(self) -> Self {
        self.with_align_items(AlignItems::End)
    }

    /// Stretches items to fill container.
    pub fn align_items_stretch(self) -> Self {
        self.with_align_items(AlignItems::Stretch)
    }

    /// Flex start
    pub fn align_items_flex_start(self) -> Self {
        self.with_align_items(AlignItems::FlexStart)
    }

    /// Flex end
    pub fn align_items_flex_end(self) -> Self {
        self.with_align_items(AlignItems::FlexEnd)
    }

    /// Justifies content to the start.
    pub fn justify_content_start(self) -> Self {
        self.with_justify_content(JustifyContent::Start)
    }

    /// Justifies content to the end.
    pub fn justify_content_end(self) -> Self {
        self.with_justify_content(JustifyContent::End)
    }

    /// Spaces content evenly.
    pub fn justify_content_space_between(self) -> Self {
        self.with_justify_content(JustifyContent::SpaceBetween)
    }

    /// Spaces content with space around.
    pub fn justify_content_space_around(self) -> Self {
        self.with_justify_content(JustifyContent::SpaceAround)
    }

    /// Spaces content with space evenly.
    pub fn justify_content_space_evenly(self) -> Self {
        self.with_justify_content(JustifyContent::SpaceEvenly)
    }

    /// Justify content flex start
    pub fn justify_content_flex_start(self) -> Self {
        self.with_justify_content(JustifyContent::FlexStart)
    }

    /// Justify content flex end
    pub fn justify_content_flex_end(self) -> Self {
        self.with_justify_content(JustifyContent::FlexEnd)
    }

    // Gap methods
    fn with_gap(mut self, gap: Size<LengthPercentage>) -> Self {
        self.get_layout_mut().gap = gap;
        self
    }

    fn with_row_gap(mut self, gap: LengthPercentage) -> Self {
        self.get_layout_mut().gap.height = gap;
        self
    }

    fn with_column_gap(mut self, gap: LengthPercentage) -> Self {
        self.get_layout_mut().gap.width = gap;
        self
    }

    /// Sets both row and column gap.
    pub fn gap(self, gap: f32) -> Self {
        let gap_val = LengthPercentage::length(gap);
        self.with_gap(Size { width: gap_val, height: gap_val })
    }

    /// Sets horizontal and vertical gaps.
    pub fn gap_xy(self, x: f32, y: f32) -> Self {
        self.with_gap(Size {
            width: LengthPercentage::length(x),
            height: LengthPercentage::length(y),
        })
    }

    /// Sets the row gap.
    pub fn row_gap(self, gap: f32) -> Self {
        self.with_row_gap(LengthPercentage::length(gap))
    }

    /// Sets the column gap.
    pub fn column_gap(self, gap: f32) -> Self {
        self.with_column_gap(LengthPercentage::length(gap))
    }
}
