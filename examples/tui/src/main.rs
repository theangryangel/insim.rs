#[allow(dead_code)]
mod util;

use crate::util::event::{Event, Events};
use bounded_vec_deque::BoundedVecDeque;
use chrono;
use insim;
use tokio::sync::mpsc;
use std::{
    collections::HashMap,
    default::Default,
    error::Error,
    io,
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    time::Duration,
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

pub struct Player {
    pub name: String,
    pub lap: u16,
    pub vehicle: String,
    pub position: u8,
}

pub struct StatefulTable {
    state: TableState,
}

impl StatefulTable {
    fn new() -> StatefulTable {
        StatefulTable {
            state: TableState::default(),
        }
    }

    pub fn next(&mut self) {
    }

    pub fn previous(&mut self) {
    }
}

#[derive(Clone)]
struct ClientState {
    players: Arc<Mutex<HashMap<u8, Player>>>,
    chat: Arc<Mutex<BoundedVecDeque<String>>>,
    update: mpsc::UnboundedSender<bool>,
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

    let (update_tx, mut update_rx) = mpsc::unbounded_channel();

    let chat = Arc::new(Mutex::new(BoundedVecDeque::new(20)));
    let players = Arc::new(Mutex::new(HashMap::new()));
    let mut table = StatefulTable::new();

    let mut client = insim::framework::Config::default()
        .relay()
        .build_with_state(ClientState {
            players: players.clone(),
            update: update_tx,
            chat: chat.clone(),
        });

    client.on_connect(|ctx| {
        ctx.send(insim::protocol::relay::HostListRequest::default().into());

        ctx.send(
            insim::protocol::relay::HostSelect {
                hname: "Nubbins AU Demo".into(),
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
        );
    });

    client.on_new_player(|ctx, data| {
        ctx.state.players.lock().unwrap().insert(data.plid, Player {
            name: data.pname.to_string(),
            lap: 0,
            vehicle: data.cname.to_string(),
            position: 0,
        });

        ctx.state.update.send(true).unwrap();
    });

    client.on_player_left(|ctx, data| {
        if let Some(_) = ctx.state.players.lock().unwrap().remove(&data.plid) {
            ctx.state.update.send(true).unwrap();
        }
    });

    client.on_lap(|ctx, data| {
        if let Some(player) = ctx.state.players.lock().unwrap().get_mut(&data.plid) {
            player.lap = data.lapsdone;

            ctx.state.chat.lock().unwrap().push_front(format!(
                "{}: {} finished lap {}",
                chrono::Local::now().format("%H:%M:%S"),
                player.name,
                data.lapsdone
            ));
    
            ctx.state.update.send(true).unwrap();
        }
    });

    client.on_multi_car_info(|ctx, data| {
        let mut any = false;

        for info in data.info.iter() {
            if let Some(player) = ctx.state.players.lock().unwrap().get_mut(&info.plid) {
                player.lap = info.lap;
                player.position = info.position;
                any = true;
            }
        }

        if any {
            ctx.state.update.send(true).unwrap();
        }
    });

    client.on_split(|ctx, data| {
        if let Some(player) = ctx.state.players.lock().unwrap().get(&data.plid) {
            ctx.state.chat.lock().unwrap().push_front(format!(
                "{}: {} spx {}",
                chrono::Local::now().format("%H:%M:%S"),
                player.name,
                data.etime,
            ));

            ctx.state.update.send(true).unwrap();
        }
    });

    client.on_message(|ctx, data| {
        ctx.state.chat.lock().unwrap().push_front(format!(
            "{}: {}", chrono::Local::now().format("%H:%M:%S"),
            insim::string::colours::strip(data.msg.to_string())
        ));

        ctx.state.update.send(true).unwrap();
    });

    // TODO: not handling client shutdown
    // TODO: discuss if the framework API is actually useful outside of "toy" examples
    tokio::spawn(async move {
        client.run().await
    });

    loop {
        tokio::select! {
            event = events.next() => {
                match event? {
                    Event::Input(key) => {
                        match key {
                            Key::Char('q') => {
                                break;
                            },
                            Key::Char('n') => {
                                table.next();
                            },
                            Key::Char('p') => {
                                table.previous();
                            },
                            _ => {
                                continue;
                            }
                        }
                    },
                    _ => {
                        continue;
                    }
                }
            },
            should_update = update_rx.recv() => {
                if should_update != Some(true) {
                    continue;
                }
            }
        };

        terminal.draw(|f| {
            let rects = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());

            // Players
            let selected_style = Style::default().add_modifier(Modifier::REVERSED);
            let header_cells = ["#", "Player", "Car", "Lap", "Time"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
            let header = Row::new(header_cells).height(1);

            let row_data = players.lock().unwrap();

            let rows = row_data.iter().map(|(_key, item)| {
                let cells = vec![
                    Cell::from(Span::raw(item.position.to_string())),
                    Cell::from(Span::raw(&item.name)),
                    Cell::from(Span::raw(&item.vehicle)),
                    Cell::from(Span::raw(item.lap.to_string())),
                ];
                Row::new(cells).height(1)
            });

            let t = Table::new(rows)
                .header(header)
                .block(Block::default().borders(Borders::ALL).title("Players"))
                .highlight_style(selected_style)
                .highlight_symbol("* ")
                .widths(&[
                    Constraint::Length(3),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ]);
            f.render_stateful_widget(t, rects[0], &mut table.state);

            // Chat
            let chat_items: Vec<ListItem> = chat
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
                );
            f.render_widget(items, rects[1]);
        })?;
    }

    Ok(())
}
