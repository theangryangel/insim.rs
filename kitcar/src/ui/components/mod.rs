//! Components
use super::{Element, Styled};

/// Vertical stack - arranges children vertically
pub fn vstack<I>(children: I) -> Element
where
    I: IntoIterator<Item = Element>,
{
    let mut container = Element::container().flex().flex_col();

    for child in children {
        container = container.with_child(child);
    }

    container
}

/// Horizontal stack - arranges children horizontally
pub fn hstack<I>(children: I) -> Element
where
    I: IntoIterator<Item = Element>,
{
    let mut container = Element::container().flex().flex_row();

    for child in children {
        container = container.with_child(child);
    }

    container
}

/// Z stack - overlapping elements (absolute positioning)
pub fn zstack<I>(children: I) -> Element
where
    I: IntoIterator<Item = Element>,
{
    let mut container = Element::container();
    // Note: You might need to add position: relative/absolute to your Styled trait

    for child in children {
        container = container.with_child(child);
    }

    container
}

/// Centered container - centers all children both horizontally and vertically
pub fn centered<I>(children: I) -> Element
where
    I: IntoIterator<Item = Element>,
{
    let mut container = Element::container().flex().justify_center().items_center();

    for child in children {
        container = container.with_child(child);
    }

    container
}

/// Spacer - takes up available space (useful in flex layouts)
pub fn spacer() -> Element {
    Element::container().flex_grow(1.0)
}

/// Fixed spacer with specific dimensions
pub fn spacer_fixed(width: f32, height: f32) -> Element {
    Element::container().w(width).h(height)
}

/// Card-like container with padding and defined size
pub fn card<I>(width: f32, height: f32, children: I) -> Element
where
    I: IntoIterator<Item = Element>,
{
    let mut container = Element::container()
        .w(width)
        .h(height)
        .p(20.0)
        .flex()
        .flex_col();

    for child in children {
        container = container.with_child(child);
    }

    container
}

/// Toolbar - horizontal row of buttons/items with spacing
pub fn toolbar<I>(items: I) -> Element
where
    I: IntoIterator<Item = Option<Element>>,
{
    Element::container()
        .flex()
        .flex_row()
        .items_center()
        .p(10.0)
        .with_children(items)
}

/// Dialog/modal container - centered with fixed size
pub fn dialog<I>(width: f32, height: f32, children: I) -> Element
where
    I: IntoIterator<Item = Element>,
{
    centered([card(width, height, children)])
}
