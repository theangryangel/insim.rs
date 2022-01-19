use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};

use crate::style::bold;

#[derive(Debug, Default)]
pub(crate) struct ServersTable {
    data: Vec<(insim::protocol::relay::HostInfo, String)>,
    state: TableState,
}

impl ServersTable {
    pub fn on_network(&mut self, e: &insim::client::Event) {
        match e {
            insim::client::Event::Packet(frame) => match frame {
                insim::protocol::Packet::RelayHostList(insim::protocol::relay::HostList {
                    hinfo,
                    ..
                }) => {
                    for info in hinfo.iter() {
                        self.push(
                            info.to_owned(),
                            insim::string::colours::strip(info.hname.to_lossy_string()),
                        );

                        if info
                            .flags
                            .contains(insim::protocol::relay::HostInfoFlags::LAST)
                        {
                            self.sort();
                        }
                    }
                }

                _ => {}
            },

            _ => {}
        }
    }

    pub fn selected(&self) -> Option<&insim::string::CodepageString> {
        if let Some(selected) = self.state.selected() {
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

    pub fn render<B: tui::backend::Backend>(
        &mut self,
        frame: &mut tui::terminal::Frame<B>,
        area: tui::layout::Rect,
    ) {
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
        frame.render_widget(help, inner[0]);

        // table
        let header_cells = ["Server", "Track", "Connections"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1);

        let rows = self.data.iter().map(|(hinfo, name)| {
            let cells = vec![
                Cell::from(Span::raw(name)),
                Cell::from(Span::raw(hinfo.track.to_string())),
                Cell::from(Span::raw(hinfo.numconns.to_string())),
            ];
            Row::new(cells).height(1)
        });

        let t = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Servers"))
            .highlight_symbol("* ")
            .widths(&[
                Constraint::Min(100),
                Constraint::Length(10),
                Constraint::Length(15),
            ]);

        frame.render_stateful_widget(t, inner[1], &mut self.state);
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn scroll_next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn scroll_prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
