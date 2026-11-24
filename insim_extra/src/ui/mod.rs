//! Button UIs built from views.
//!
//! A view is a function from state to a tree of [`Node`]s. [`Canvas`] lays the
//! tree out and diffs it against what each player already has, so only changed
//! buttons go over the wire. A [`Component`] is a self-contained piece with its
//! own per-player state: place it with [`Ui::component`] and a [`Screen`] keeps
//! one per player. Components nest, and their clicks go straight to them.
//!
//! kitcar's `Host` drives a service's `view` for you, or drive a [`Screen`]
//! yourself.
//!
//! # Effects
//!
//! A component's [`mounted`](Component::mounted) and
//! [`update`](Component::update) return [`Effects`]: timers, tasks and streams
//! whose messages come back to that instance, and what to
//! [`emit`](Effects::emit) to its parent. Load a page from the database as the
//! component mounts, refetch when a button is clicked, close a toast after a
//! few seconds. The work runs on Tokio and is aborted when the instance
//! unmounts, so a closed dialog never hears back from its query. Data comes in
//! through props; clone what a task needs (a pool, an id) into it.
//!
//! Anything outside the UI, such as sending packets, is the service's job:
//! emit, and let [`Child::on`] turn it into a service message.
//!
//! Countdowns and clocks that only *show* the time don't need timers: draw
//! them from [`Ui::now`] and ask for the next frame with [`Ui::redraw_at`] or
//! [`Ui::countdown`]. Use a timer when something should *change* at a time.
//!
//! kitcar's `Host` routes results back for you. Driving a [`Screen`] yourself,
//! make it [`Screen::with_outbox`] and pass each [`Delivery`] to
//! [`Screen::deliver`].
//!
//! Sizes and offsets are UI units on LFS's 200 x 200 button grid. Numeric
//! builders take integers or floats, so `.w(30).h(6).p(2).top(5)` works.
//!
//! # Positioning
//!
//! Nodes are static by default, so insets do nothing. Call
//! [`Node::position_relative`] to apply them, or to anchor absolute
//! descendants. [`Node::position_absolute`] and [`Node::position_fixed`] take a
//! node out of flow; fixed nodes use the full 200 x 200 viewport, even when the
//! root is smaller or offset.
//!
//! # Layout builders
//!
//! Layout is Taffy, so it follows CSS block, flex and grid rules. Reach for
//! `with_style` when a builder is missing. `gap_x` and `gap_y` stay horizontal and vertical
//! whatever the flex direction.
//!
//! Percentage heights need a definite height on the container; an `auto`
//! height gives `h_percent` nothing to work from. Content sizing (`fit_content` and
//! friends) looks at children only: button text is not measured.
//!
//! Toggle a group with `.when(condition, |node| node.with_children(...))`.
//! Children built inside the closure are skipped when the condition is false.
//!
//! # Multiplayer only
//!
//! Screens are for dedicated servers: kitcar never draws for the local
//! connection ([`ConnectionId::LOCAL`](insim::identifiers::ConnectionId::LOCAL) / UCID 0). For a local
//! tool, drive a [`Canvas`] yourself.

pub mod canvas;
mod component;
pub mod components;
mod effects;
pub mod id_pool;
mod node;

pub use canvas::{Canvas, CanvasDiff};
pub use component::{Child, Component, Delivery, Key, Outbox, Screen, Ui};
pub use effects::Effects;
pub use node::{Action, Node};

/// Empty container, handy as a layout root.
pub fn container<Msg>() -> node::Node<Msg> {
    node::Node::container()
}

/// Horizontal flex container: `container().flex_row().with_children(children)`.
pub fn row<Msg>(children: impl IntoIterator<Item = node::Node<Msg>>) -> node::Node<Msg> {
    node::Node::container()
        .flex()
        .flex_row()
        .with_children(children)
}

/// Vertical flex container: `container().flex_col().with_children(children)`.
pub fn col<Msg>(children: impl IntoIterator<Item = node::Node<Msg>>) -> node::Node<Msg> {
    node::Node::container()
        .flex()
        .flex_col()
        .with_children(children)
}

/// Filled background with no text.
pub fn background<Msg>(bstyle: insim::insim::BtnStyle) -> node::Node<Msg> {
    node::Node::background(bstyle)
}

/// Button that sends `msg` when clicked.
pub fn clickable<Msg>(
    text: impl Into<String>,
    bstyle: insim::insim::BtnStyle,
    msg: Msg,
) -> node::Node<Msg> {
    node::Node::clickable(text, bstyle, msg)
}

/// Text label.
pub fn text<Msg>(text: impl Into<String>, bstyle: insim::insim::BtnStyle) -> node::Node<Msg> {
    node::Node::text(text, bstyle)
}

/// Type-in button; `mapper` turns the typed text into a message.
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

/// Renders nothing.
pub fn empty<Msg>() -> node::Node<Msg> {
    node::Node::empty()
}
