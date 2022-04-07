use std::borrow::Cow;

use tui::layout::Rect;
use tui::widgets::TableState;

#[derive(Debug, Default)]
pub struct ServersState {
    pub data: Vec<(insim::protocol::relay::HostInfo, String)>,
    pub inner: TableState,
}

impl ServersState {
    pub fn on_network(&mut self, e: &insim::client::Event) {
        if let insim::client::Event::Frame(insim::protocol::Packet::RelayHostList(
            insim::protocol::relay::HostList { hinfo, .. },
        )) = e
        {
            for info in hinfo.iter() {
                self.push(info.to_owned(), info.hname.to_lossy_string());

                if info
                    .flags
                    .contains(insim::protocol::relay::HostInfoFlags::LAST)
                {
                    self.sort();
                }
            }
        }
    }

    pub fn selected(&self) -> Option<&insim::string::CodepageString> {
        if let Some(selected) = self.inner.selected() {
            return Some(&self.data[selected].0.hname);
        }

        None
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn push(&mut self, hinfo: insim::protocol::relay::HostInfo, name: String) {
        self.data.push((hinfo, name));
    }

    pub fn sort(&mut self) {
        self.data
            .sort_by(|(a, _), (b, _)| b.numconns.partial_cmp(&a.numconns).unwrap());
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn scroll_next(&mut self) {
        let i = match self.inner.selected() {
            Some(i) => {
                if i >= self.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.inner.select(Some(i));
    }

    pub fn scroll_prev(&mut self) {
        let i = match self.inner.selected() {
            Some(i) => {
                if i == 0 {
                    self.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.inner.select(Some(i));
    }
}

use tui::widgets::{StatefulWidget, Widget};

use tui::{
    layout::{Constraint, Direction, Layout},
    text::{Span, Spans, Text},
    widgets::{Cell, Paragraph, Row, Table},
};

use tui::{
    style::{Modifier, Style},
    widgets::{Block, Borders},
};

use crate::style::bold;
use crate::view::colourify;

#[derive(Default)]
pub struct ServersWidget {}

impl StatefulWidget for ServersWidget {
    type State = ServersState;

    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer, state: &mut Self::State) {
        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
            .split(area);

        let text = Text::from(Spans::from(vec![
            Span::raw("controls: "),
            bold("\u{2191}\u{2193}"), // up - down
            Span::raw(" = scroll, "),
            bold("\u{21B5}"), // enter
            Span::raw(" = select server, "),
            Span::raw("r = refresh, "),
            bold("q"),
            Span::raw(" or "),
            bold("\u{238B} esc"),
            Span::raw(" = back"),
        ]));

        let help = Paragraph::new(text);
        help.render(inner[0], buf);

        // table
        let header_cells = ["Server", "Track", "Connections"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1);

        let rows = state.data.iter().map(|(hinfo, name)| {
            let cells = vec![
                Cell::from(colourify(Cow::Borrowed(name))),
                Cell::from(Span::raw(hinfo.track.to_string())),
                Cell::from(Span::raw(hinfo.numconns.to_string())),
            ];
            Row::new(cells).height(1)
        });

        let t = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Servers"))
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .widths(&[
                Constraint::Min(32),
                Constraint::Length(10),
                Constraint::Length(15),
            ]);
        StatefulWidget::render(t, inner[1], buf, &mut state.inner);
    }
}
