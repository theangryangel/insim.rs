use itertools::Itertools;
use std::borrow::Cow;
use std::collections::HashMap;

use tui::layout::Rect;
use tui::widgets::TableState;

#[derive(Default)]
pub struct Player {
    name: String,
    lap: u16,
    position: u8,
}

#[derive(Default)]
pub struct PlayerListState {
    pub inner: HashMap<u8, Player>,
    pub table_state: TableState,
}

impl PlayerListState {
    pub fn on_network(&mut self, e: &insim::client::Event) {
        match e {
            insim::client::Event::Packet(insim::protocol::Packet::NewPlayer(frame)) => {
                self.inner.insert(
                    frame.plid,
                    Player {
                        name: frame.pname.to_lossy_string(),
                        ..Default::default()
                    },
                );
            }

            insim::client::Event::Packet(insim::protocol::Packet::PlayerLeave(frame)) => {
                self.inner.remove(&frame.plid);
            }

            insim::client::Event::Packet(insim::protocol::Packet::MultiCarInfo(frame)) => {
                for player in frame.info.iter() {
                    if let Some(p) = self.inner.get_mut(&player.plid) {
                        p.lap = player.lap;
                        p.position = player.position;
                    }
                }
            }

            _ => {}
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn scroll_next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn scroll_prev(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
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
pub struct PlayerListWidget {}

impl StatefulWidget for PlayerListWidget {
    type State = PlayerListState;

    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer, state: &mut Self::State) {
        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
            .split(area);

        // Controls
        let text = Text::from(Spans::from(vec![
            Span::raw("controls: "),
            bold("\u{2191}\u{2193}"), // up - down
            Span::raw(" = scroll, "),
            bold("\u{21B5}"), // enter
            Span::raw(" = select player, "),
            bold("q"),
            Span::raw(" or "),
            bold("\u{238B} esc"),
            Span::raw(" = back"),
        ]));

        let help = Paragraph::new(text);
        help.render(inner[0], buf);

        // table
        let header_cells = ["#", "ID", "Player", "Lap"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1);

        let rows = state
            .inner
            .iter()
            .sorted_by(|(plida, playera), (plidb, playerb)| {
                playera.position.partial_cmp(&playerb.position).unwrap()
            })
            .map(|(plid, player)| {
                let cells = vec![
                    Cell::from(format!("{}", player.position)),
                    Cell::from(format!("{}", plid)),
                    Cell::from(colourify(Cow::Borrowed(&player.name))),
                    Cell::from(format!("{}", player.lap)),
                ];
                Row::new(cells).height(1)
            });

        let t = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Players"))
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .widths(&[
                Constraint::Min(3),
                Constraint::Min(3),
                Constraint::Length(20),
                Constraint::Length(5),
            ]);
        StatefulWidget::render(t, inner[1], buf, &mut state.table_state);
    }
}
