use tui::layout::Alignment;
use tui::text::{Span, Spans};
use tui::widgets::{Paragraph, Widget};
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
};

#[derive(Default)]
pub struct ConnectedWidget {
    connected: bool,
}

impl ConnectedWidget {
    pub fn connected(mut self, state: bool) -> Self {
        self.connected = state;
        self
    }
}

impl Widget for ConnectedWidget {
    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
        let lines = if self.connected {
            Span::styled(
                "(CONNECTED)",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                "(DISCONNECTED)",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )
        };

        let state = Paragraph::new(Spans::from(vec![Span::raw("Insim Relay: "), lines]))
            .alignment(Alignment::Left);

        state.render(area, buf);
    }
}
