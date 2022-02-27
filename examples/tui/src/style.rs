use std::borrow::Cow;
use tui::{
    style::{Modifier, Style},
    text::Span,
};

pub fn bold<'a, I: Into<Cow<'a, str>>>(text: I) -> Span<'a> {
    Span::styled(text, Style::default().add_modifier(Modifier::BOLD))
}
