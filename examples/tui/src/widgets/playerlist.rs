use itertools::Itertools;
use std::borrow::Cow;
use std::collections::HashMap;

use tui::layout::Rect;
use tui::widgets::TableState;

#[derive(Default)]
pub struct Connection {
    uname: String,
    name: String,
    lap: Option<u16>,
    position: Option<u8>,
    lap_time: Option<u32>,
    elapsed_time: Option<u32>,
}

impl Connection {
    pub fn get_lap_string(&self) -> String {
        if let Some(lap) = self.lap {
            format!("{}", lap)
        } else {
            String::from("-")
        }
    }

    pub fn get_position_string(&self) -> String {
        if let Some(position) = self.position {
            format!("{}", position)
        } else {
            String::from("-")
        }
    }

    pub fn get_lap_time_string(&self) -> String {
        if let Some(lap_time) = &self.lap_time {
            lap_time.to_string()
        } else {
            String::from("-")
        }
    }

    pub fn get_elapsed_time_string(&self) -> String {
        if let Some(elapsed_time) = &self.elapsed_time {
            elapsed_time.to_string()
        } else {
            String::from("-")
        }
    }
}

#[derive(Default)]
pub struct ConnectionListState {
    pub inner: HashMap<u8, Connection>,
    pub plid_ucid: HashMap<u8, u8>,

    pub table_state: TableState,
}

impl ConnectionListState {
    pub fn on_network(&mut self, e: &insim::client::Event) {
        match e {
            insim::client::Event::Frame(insim::protocol::Packet::NewConnection(frame)) => {
                let connection = Connection {
                    uname: frame.uname.to_string(),
                    name: frame.pname.to_string(),

                    ..Default::default()
                };
                self.inner.insert(frame.ucid, connection);
            }

            insim::client::Event::Frame(insim::protocol::Packet::ConnectionLeave(frame)) => {
                self.inner.remove(&frame.ucid);
            }

            insim::client::Event::Frame(insim::protocol::Packet::NewPlayer(frame)) => {
                self.plid_ucid.insert(frame.plid, frame.ucid);
            }

            insim::client::Event::Frame(insim::protocol::Packet::PlayerLeave(frame)) => {
                self.plid_ucid.remove(&frame.plid);
            }

            insim::client::Event::Frame(insim::protocol::Packet::MultiCarInfo(frame)) => {
                for player in frame.info.iter() {
                    let ucid = self.plid_ucid.get(&player.plid);
                    if ucid.is_none() {
                        continue;
                    }
                    let ucid = ucid.unwrap();

                    if let Some(p) = self.inner.get_mut(ucid) {
                        p.lap = Some(player.lap);
                        p.position = Some(player.position);
                    }
                }
            }

            insim::client::Event::Frame(insim::protocol::Packet::Lap(frame)) => {
                let ucid = self.plid_ucid.get(&frame.plid);
                if ucid.is_none() {
                    return;
                }
                let ucid = ucid.unwrap();

                if let Some(p) = self.inner.get_mut(ucid) {
                    p.lap = Some(frame.lapsdone);
                    p.lap_time = Some(frame.ltime);
                    p.elapsed_time = Some(frame.etime);
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
    type State = ConnectionListState;

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
        let header_cells = [
            "#",
            "UCID",
            "User",
            "Player",
            "Lap",
            "Lap Time",
            "Elapsed Time",
        ]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1);

        let rows = state
            .inner
            .iter()
            .sorted_by(|(_plida, playera), (_plidb, playerb)| {
                if playerb.position.is_none() {
                    return std::cmp::Ordering::Less;
                }
                if playera.position.is_none() {
                    return std::cmp::Ordering::Greater;
                }
                playera.position.partial_cmp(&playerb.position).unwrap()
            })
            .map(|(ucid, player)| {
                let cells = vec![
                    Cell::from(player.get_position_string()),
                    Cell::from(format!("{}", ucid)),
                    Cell::from(player.uname.to_owned()),
                    Cell::from(colourify(Cow::Borrowed(&player.name))),
                    Cell::from(player.get_lap_string()),
                    Cell::from(player.get_lap_time_string()),
                    Cell::from(player.get_elapsed_time_string()),
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
                Constraint::Min(4),
                Constraint::Length(32),
                Constraint::Length(32),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
            ]);
        StatefulWidget::render(t, inner[1], buf, &mut state.table_state);
    }
}
