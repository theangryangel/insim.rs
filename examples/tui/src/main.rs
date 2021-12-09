#[allow(dead_code)]
mod util;

use crate::util::event::{Event, Events};
use bounded_vec_deque::BoundedVecDeque;
use chrono;
use insim;
use std::{
    collections::HashMap,
    error::Error,
    io,
    sync::{Arc, Mutex},
};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, List, ListItem, Row, Table, TableState},
    Terminal,
};

pub struct StatefulTable {
    state: TableState,
    pub players: Arc<Mutex<HashMap<u8, insim::protocol::insim::Npl>>>,
    pub chat: Arc<Mutex<BoundedVecDeque<String>>>,
}

impl StatefulTable {
    fn new() -> StatefulTable {
        StatefulTable {
            state: TableState::default(),
            players: Arc::new(Mutex::new(HashMap::new())),
            chat: Arc::new(Mutex::new(BoundedVecDeque::new(20))),
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.players.lock().unwrap().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.players.lock().unwrap().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

struct Party {
    pub players: Arc<Mutex<HashMap<u8, insim::protocol::insim::Npl>>>,
    chat: Arc<Mutex<BoundedVecDeque<String>>>,
}

impl insim::framework::EventHandler for Party {
    fn on_connect(&self, ctx: &insim::framework::Client) {
        ctx.send(insim::protocol::relay::HostListRequest::default().into());

        ctx.send(
            insim::protocol::relay::HostSelect {
                hname: "^1(^3FM^1) ^4GTi Thursday".into(),
                ..Default::default()
            }
            .into(),
        );

        ctx.send(
            insim::protocol::insim::Tiny {
                reqi: 0,
                subtype: insim::protocol::insim::TinyType::Npl,
            }
            .into(),
        )
    }

    fn on_new_player(
        &self,
        _client: &insim::framework::Client,
        data: &insim::protocol::insim::Npl,
    ) {
        self.players.lock().unwrap().insert(data.plid, data.clone());
    }

    fn on_message(&self, _client: &insim::framework::Client, data: &insim::protocol::insim::Mso) {
        self.chat.lock().unwrap().push_front(format!(
            "{}: {}",
            chrono::Local::now().format("%H:%M:%S"),
            insim::string::colours::strip(data.msg.to_string())
        ));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let mut table = StatefulTable::new();

    let client = insim::framework::Config::default()
        .relay()
        .using_event_handler(Party {
            players: table.players.clone(),
            chat: table.chat.clone(),
        })
        .build();

    tokio::spawn(client.run());
    //let res = client.run().await;

    // Input
    loop {
        terminal.draw(|f| {
            let rects = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());

            // Players
            let selected_style = Style::default().add_modifier(Modifier::REVERSED);
            let header_cells = ["Player", "Car", "Lap"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
            let header = Row::new(header_cells).height(1);
            let players = table.players.lock().unwrap();
            let rows = players.iter().map(|(_key, item)| {
                let cells = vec![
                    Cell::from(Span::raw(item.pname.to_string())),
                    Cell::from(Span::raw(item.cname.to_string())),
                ];
                Row::new(cells).height(1 as u16)
            });
            let t = Table::new(rows)
                .header(header)
                .block(Block::default().borders(Borders::ALL).title("Players"))
                .highlight_style(selected_style)
                .highlight_symbol("* ")
                .widths(&[
                    Constraint::Percentage(50),
                    Constraint::Length(30),
                    Constraint::Max(10),
                ]);
            f.render_stateful_widget(t, rects[0], &mut table.state);

            // Chat
            let chat_items: Vec<ListItem> = table
                .chat
                .lock()
                .unwrap()
                .iter()
                .map(|item| ListItem::new(item.to_string()))
                .collect();

            let items = List::new(chat_items)
                .block(Block::default().borders(Borders::ALL).title("Chat"))
                .highlight_style(
                    Style::default()
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">");
            f.render_widget(items, rects[1]);
        })?;

        if let Event::Input(key) = events.next()? {
            match key {
                Key::Char('q') => {
                    break;
                }
                Key::Down => {
                    table.next();
                }
                Key::Up => {
                    table.previous();
                }
                _ => {}
            }
        };
    }

    Ok(())
}
