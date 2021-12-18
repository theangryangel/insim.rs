#[allow(dead_code)]
mod util;

use crate::util::event::{Event, Events};
use bounded_vec_deque::BoundedVecDeque;
use chrono;
use insim;
use std::{
    collections::HashMap,
    default::Default,
    error::Error,
    io,
    sync::{Arc, Mutex},
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

#[derive(Default)]
pub struct Connection {
    pub user_name: String,
    pub player_name: insim::string::CodepageString,

    pub on_track: bool,

    pub finish_position: Option<u8>,
    pub current_race_lap: Option<u16>,

    pub best_lap_time: Option<u32>,
    pub last_lap_time: Option<u32>,

    pub flags: Option<insim::protocol::insim::PlayerFlags>,
    pub vehicle: Option<insim::vehicle::Vehicle>,
    pub plate: Option<insim::string::CodepageString>,
    pub tyres: Option<Vec<insim::protocol::insim::TyreCompound>>,

    pub handicap_mass: Option<u8>,
    pub handicap_intake_restriction: Option<u8>,
}

#[derive(Clone)]
pub struct GameState {
    /// map of connections
    /// this represents both players and connections
    connections: Arc<Mutex<HashMap<u8, Connection>>>,

    /// map of plid to connid
    players: Arc<Mutex<HashMap<u8, u8>>>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            connections: Arc::new(Mutex::new(HashMap::new())),
            players: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_connection(&self, data: &insim::protocol::insim::Ncn) {
        self.connections.lock().unwrap().insert(
            data.ucid,
            Connection {
                user_name: data.uname.clone(),
                player_name: data.pname.clone(),
                on_track: false,

                ..Default::default()
            },
        );
    }

    pub fn remove_connection(&self, data: &insim::protocol::insim::Cnl) {
        self.connections.lock().unwrap().remove(&data.ucid);
    }

    pub fn add_player(&self, data: &insim::protocol::insim::Npl) {
        if let Some(conn) = self.connections.lock().unwrap().get_mut(&data.ucid) {
            conn.on_track = true;
            conn.player_name = data.pname.clone();
            conn.finish_position = None;
            conn.flags = Some(data.flags.clone());
            conn.vehicle = Some(data.cname.clone());
            conn.plate = Some(data.plate.clone());
            conn.tyres = Some(data.tyres.clone());
            conn.handicap_mass = Some(data.h_mass.clone());
            conn.handicap_intake_restriction = Some(data.h_tres.clone());
        }

        self.players.lock().unwrap().insert(data.plid, data.ucid);
    }

    pub fn remove_player(&self, data: &insim::protocol::insim::Pll) {
        let ucid = self.players.lock().unwrap().remove(&data.plid);

        if ucid.is_none() {
            return;
        }

        let ucid = ucid.unwrap();

        if let Some(conn) = self.connections.lock().unwrap().get_mut(&ucid) {
            conn.on_track = false;
            conn.finish_position = None;
            conn.flags = None;
            conn.vehicle = None;
            conn.plate = None;
            conn.tyres = None;
            conn.handicap_mass = None;
            conn.handicap_intake_restriction = None;
        }
    }

    pub fn update_player_info(&self, data: &insim::protocol::insim::CompCar) {
        let ucid = self.players.lock().unwrap().get(&data.plid).cloned();

        if ucid.is_none() {
            return;
        }

        let ucid = ucid.unwrap();

        if let Some(player) = self.connections.lock().unwrap().get_mut(&ucid) {
            player.current_race_lap = Some(data.lap.clone());
        }
    }

    pub fn update_player_lap(&self, data: &insim::protocol::insim::Lap) {
        let ucid = self.players.lock().unwrap().get(&data.plid).cloned();

        if ucid.is_none() {
            return;
        }

        let ucid = ucid.unwrap();

        if let Some(player) = self.connections.lock().unwrap().get_mut(&ucid) {
            player.current_race_lap = Some(data.lapsdone.clone());
            player.last_lap_time = Some(data.ltime.clone());

            if player.best_lap_time.is_none() || player.best_lap_time.unwrap() > data.ltime {
                player.best_lap_time = Some(data.ltime.clone());
            }
        }
    }

    pub fn players(&self) -> Arc<Mutex<HashMap<u8, Connection>>> {
        self.connections.clone()
    }
}

pub struct StatefulTable {
    state: TableState,

    pub game: GameState,
    pub chat: Arc<Mutex<BoundedVecDeque<String>>>,
}

impl StatefulTable {
    fn new() -> StatefulTable {
        StatefulTable {
            state: TableState::default(),
            game: GameState::new(),
            chat: Arc::new(Mutex::new(BoundedVecDeque::new(20))),
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.game.players().lock().unwrap().len() - 1 {
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
                    self.game.players().lock().unwrap().len() - 1
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
    game: GameState,
    chat: Arc<Mutex<BoundedVecDeque<String>>>,
}

impl insim::framework::EventHandler for Party {
    fn on_connect(&self, ctx: &insim::framework::Client) {
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
                subtype: insim::protocol::insim::TinyType::Ncn,
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
    }

    fn on_new_player(
        &self,
        _client: &insim::framework::Client,
        data: &insim::protocol::insim::Npl,
    ) {
        self.game.add_player(&data);
    }

    fn on_player_left(
        &self,
        _client: &insim::framework::Client,
        data: &insim::protocol::insim::Pll,
    ) {
        self.game.remove_player(&data);
    }

    fn on_new_connection(
        &self,
        _client: &insim::framework::Client,
        data: &insim::protocol::insim::Ncn,
    ) {
        self.game.add_connection(&data);
    }

    fn on_connection_left(
        &self,
        _client: &insim::framework::Client,
        data: &insim::protocol::insim::Cnl,
    ) {
        self.game.remove_connection(&data);
    }

    fn on_message(&self, _client: &insim::framework::Client, data: &insim::protocol::insim::Mso) {
        self.chat.lock().unwrap().push_front(format!(
            "{}: {}",
            chrono::Local::now().format("%H:%M:%S"),
            insim::string::colours::strip(data.msg.to_string())
        ));
    }

    fn on_multi_car_info(
        &self,
        _client: &insim::framework::Client,
        data: &insim::protocol::insim::Mci,
    ) {
        for info in data.info.iter() {
            self.game.update_player_info(&info);
        }
    }

    fn on_lap(&self, _client: &insim::framework::Client, data: &insim::protocol::insim::Lap) {
        self.game.update_player_lap(&data);
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
            game: table.game.clone(),
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
            let header_cells = ["Player", "Car", "Lap", "Time"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
            let header = Row::new(header_cells).height(1);

            let rows = if let Ok(players) = table.game.players().lock() {
                players
                    .iter()
                    .map(|(_key, item)| {
                        let vehicle = if let Some(vehicle) = &item.vehicle {
                            vehicle.to_string()
                        } else {
                            "".to_string()
                        };

                        let lap = if let Some(lap) = item.current_race_lap {
                            lap.to_string()
                        } else {
                            "".to_string()
                        };

                        let ltime = if let Some(ltime) = item.last_lap_time {
                            ltime.to_string()
                        } else {
                            "".to_string()
                        };

                        let cells = vec![
                            Cell::from(Span::raw(item.player_name.to_string())),
                            Cell::from(Span::raw(vehicle)),
                            Cell::from(Span::raw(lap)),
                            Cell::from(Span::raw(ltime)),
                        ];
                        Row::new(cells).height(1 as u16)
                    })
                    .collect()
            } else {
                Vec::new()
            };

            let t = Table::new(rows)
                .header(header)
                .block(Block::default().borders(Borders::ALL).title("Players"))
                .highlight_style(selected_style)
                .highlight_symbol("* ")
                .widths(&[
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
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
