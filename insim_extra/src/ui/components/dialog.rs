//! [`Dialog`]: the shared panel shell. A title, a body, and a row of buttons
//! at the foot, on one dark background.

use super::style;
use crate::ui::Node;

/// Default width, in UI units of the 200 x 200 screen.
const WIDTH: u16 = 100;
/// Padding inside the panel.
const PAD: u16 = 2;
const TITLE: u16 = 6;
const LINE: u16 = 5;
const FOOTER: u16 = 6;

/// A panel with a title, body lines and footer buttons, spaced alike wherever
/// it's used. It draws only the panel: place it yourself, say with
/// `.position_absolute().top(50).left(70)`.
///
/// ```ignore
/// ui::dialog("Top drivers")
///     .width(60)
///     .aside(format!("page {} of {pages}", page + 1))
///     .lines(rows)
///     .button(ui::clickable("<", style::panel(), Msg::Prev).w(10))
///     .button(ui::clickable("Close", style::action(), Msg::Close))
///     .button(ui::clickable(">", style::panel(), Msg::Next).w(10))
///     .build()
/// ```
///
/// Footer buttons are made full height. Any without a width share the space
/// the others leave.
#[must_use = "build it into a node"]
#[derive(Debug)]
pub struct Dialog<M> {
    title: String,
    aside: Option<String>,
    width: u16,
    body: Vec<Node<M>>,
    footer: Vec<Node<M>>,
}

/// Start a [`Dialog`] headed `title`.
pub fn dialog<M>(title: impl Into<String>) -> Dialog<M> {
    Dialog {
        title: title.into(),
        aside: None,
        width: WIDTH,
        body: Vec::new(),
        footer: Vec::new(),
    }
}

impl<M> Dialog<M> {
    /// The panel's width, padding included. 100 by default.
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Muted text at the right of the title, such as a page count.
    pub fn aside(mut self, text: impl Into<String>) -> Self {
        self.aside = Some(text.into());
        self
    }

    /// A line of text, the full width of the body.
    pub fn line(mut self, text: impl Into<String>) -> Self {
        let node = Node::text(text, style::active().align_left())
            .w(self.inner())
            .h(LINE);
        self.body.push(node);
        self
    }

    /// Lines of text, in order.
    pub fn lines<S: Into<String>>(self, lines: impl IntoIterator<Item = S>) -> Self {
        lines.into_iter().fold(self, Self::line)
    }

    /// Anything else in the body, sized by the caller. The body is
    /// [`inner_width`](Self::inner_width) wide.
    pub fn child(mut self, node: Node<M>) -> Self {
        self.body.push(node);
        self
    }

    /// A button in the footer, after any before it.
    pub fn button(mut self, node: Node<M>) -> Self {
        let sized = node
            .style()
            .is_some_and(|style| style.size.width != taffy::Dimension::auto());
        let node = node.h(FOOTER);
        self.footer
            .push(if sized { node } else { node.flex_grow(1) });
        self
    }

    /// The width inside the padding, for sizing [`child`](Self::child) nodes.
    pub fn inner_width(&self) -> u16 {
        self.inner()
    }

    fn inner(&self) -> u16 {
        self.width.saturating_sub(2 * PAD)
    }

    /// The finished panel.
    pub fn build(self) -> Node<M> {
        let inner = self.inner();
        let header = Node::container()
            .flex()
            .flex_row()
            .w(inner)
            .mb(1)
            .with_child(
                Node::text(self.title, style::title().align_left())
                    .flex_grow(1)
                    .h(TITLE),
            )
            .with_children(self.aside.map(|aside| {
                Node::text(aside, style::muted().align_right())
                    .w(inner / 3)
                    .h(TITLE)
            }));
        let footer = (!self.footer.is_empty()).then(|| {
            Node::container()
                .flex()
                .flex_row()
                .w(inner)
                .mt(1)
                .with_children(self.footer)
        });
        Node::background(style::panel())
            .w(self.width)
            .flex()
            .flex_col()
            .p(PAD)
            .with_child(header)
            .with_children(self.body)
            .with_children(footer)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use insim::{Packet, identifiers::ConnectionId, insim::Btn};

    use super::*;
    use crate::ui::{Screen, id_pool::ClickIdPool};

    fn drawn(node: Node<()>) -> Vec<Btn> {
        Screen::<()>::new(ConnectionId(2))
            .render(&mut ClickIdPool::new(), Instant::now(), |_| Some(node))
            .into_iter()
            .filter_map(|p| match p {
                Packet::Btn(btn) => Some(btn),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn lays_out_title_body_and_footer() {
        let buttons = drawn(
            dialog("Title")
                .width(60)
                .aside("1 of 2")
                .line("a line")
                .button(Node::clickable("<", style::panel(), ()).w(10))
                .button(Node::clickable("Close", style::action(), ()))
                .button(Node::clickable(">", style::panel(), ()).w(10))
                .build(),
        );
        let at = |text: &str| {
            buttons
                .iter()
                .find(|b| b.text == text)
                .unwrap_or_else(|| panic!("no {text:?} in {buttons:?}"))
        };
        // The background, then each piece inside its padding.
        assert_eq!((at("").l, at("").w), (0, 60));
        assert_eq!((at("Title").l, at("Title").t), (2, 2));
        assert_eq!(at("1 of 2").l + at("1 of 2").w, 58);
        assert_eq!(at("a line").w, 56);
        // The unsized button takes what the arrows leave.
        assert_eq!((at("<").w, at("Close").w, at(">").w), (10, 36, 10));
        assert_eq!(at("Close").l, 12);
    }

    #[test]
    fn without_buttons_there_is_no_footer() {
        let buttons = drawn(dialog("Just text").line("one").build());
        assert_eq!(buttons.len(), 3, "background, title and line: {buttons:?}");
    }
}
