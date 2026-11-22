//! UI component system.
//!
//! A view is a function from state to a tree of [`Node`]s. [`Canvas`] lays the
//! tree out and diffs it against what each player already has, so only changed
//! buttons are sent. Self-contained pieces with their own per-player state are
//! [`Component`]s, placed with [`Ui::component`] and kept for each player by a
//! [`Screen`]. Components nest, and clicks on their buttons go straight to them.
//!
//! kitcar's `Host` drives this for a service's `view`; a [`Screen`] can also be
//! driven directly.
//!
//! # Positioning
//!
//! Nodes default to static positioning: insets are ignored. Use
//! [`Node::position_relative`] to apply insets in normal flow or establish a
//! containing block for absolute descendants. [`Node::position_absolute`] and
//! [`Node::position_fixed`] remove a node from normal flow; fixed nodes use the
//! 200 × 200 UI viewport even when the root is smaller or offset.
//! When migrating from Taffy 0.13, explicitly select relative positioning for
//! nodes that previously relied on the default position applying insets.
//!
//! # Layout builders
//!
//! Flex direction helpers enable flex layout, including after `block()`.
//! Use `gap`, `gap_x`, and `gap_y` for spacing between items. Horizontal and
//! vertical gap axes do not change with flex direction. Use `with_style` to
//! configure additional Taffy properties without leaving the builder chain.
//! Numeric builders accept integers and floats via `Into<f64>`, for example
//! `.w(30).h(6).p(2).top(5)`. Values are converted to Taffy's `f32` layout
//! units internally; fractional and negative offsets remain supported.
//!
//! Use `.when(condition, |node| node.with_children(children))` to toggle a
//! group, constructing children inside the closure to skip building them when
//! false. `with_child_if` and `with_children_if` are deprecated in favor of
//! `when` or optional children.
//!
//! `size(width, height)` and `square(size)` set dimensions in UI units.
//! `w_percent(50)` and `h_percent(50)` mean 50% of the containing block's
//! corresponding dimension; percentage heights need a definite containing
//! height. Use `min_w`, `max_w`, `min_h`, and `max_h` for size limits in UI units.
//! `items_stretch()` fills children's automatic cross sizes; `self_auto()`
//! resets a child's alignment to follow its container.
//!
//! `fit_content()` now sets CSS `fit-content` sizing. Use `size_auto()` for
//! its pre-0.2 behavior of resetting both dimensions to `auto`. Content sizing
//! uses the layout of children; button text is not measured automatically.
//!
//! # Multiplayer only
//!
//! Screens are meant for **dedicated multiplayer servers**: kitcar never draws
//! for the host/local connection ([`ConnectionId::LOCAL`](insim::identifiers::ConnectionId::LOCAL) / UCID 0). For a local
//! tool, drive a [`Canvas`] yourself.

pub mod canvas;
mod component;
pub mod id_pool;
mod node;

pub use canvas::{Canvas, CanvasDiff};
pub use component::{Child, Component, Key, Screen, Ui};
pub use node::*;

/// Empty container node - useful as a layout root.
pub fn container<Msg>() -> node::Node<Msg> {
    node::Node::container()
}

/// Horizontal flex container holding `children`. Shorthand for
/// `container().flex().flex_row().with_children(children)`.
pub fn row<Msg>(children: impl IntoIterator<Item = node::Node<Msg>>) -> node::Node<Msg> {
    node::Node::container()
        .flex()
        .flex_row()
        .with_children(children)
}

/// Vertical flex container holding `children`. Shorthand for
/// `container().flex().flex_col().with_children(children)`.
pub fn col<Msg>(children: impl IntoIterator<Item = node::Node<Msg>>) -> node::Node<Msg> {
    node::Node::container()
        .flex()
        .flex_col()
        .with_children(children)
}

/// Filled background node (no text).
pub fn background<Msg>(bstyle: insim::insim::BtnStyle) -> node::Node<Msg> {
    node::Node::background(bstyle)
}

/// Clickable button that emits `msg` when pressed.
pub fn clickable<Msg>(
    text: impl Into<String>,
    bstyle: insim::insim::BtnStyle,
    msg: Msg,
) -> node::Node<Msg> {
    node::Node::clickable(text, bstyle, msg)
}

/// Static text node.
pub fn text<Msg>(text: impl Into<String>, bstyle: insim::insim::BtnStyle) -> node::Node<Msg> {
    node::Node::text(text, bstyle)
}

/// Typeable input node - `mapper` converts the typed string to a message.
pub fn typein<Msg, F>(
    text: impl Into<String>,
    bstyle: insim::insim::BtnStyle,
    limit: u8,
    mapper: F,
) -> node::Node<Msg>
where
    F: Fn(String) -> Msg + Send + Sync + 'static,
{
    node::Node::typein(text, bstyle, limit, mapper)
}

/// Zero-sized placeholder node.
pub fn empty<Msg>() -> node::Node<Msg> {
    node::Node::empty()
}
