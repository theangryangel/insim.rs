extern crate insim;

use std::sync::{Arc, Mutex};

use bounded_vec_deque::BoundedVecDeque;
use chrono;
use futures::{SinkExt, StreamExt};
use tracing_subscriber;

use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode, KeyEvent},
    execute, terminal,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, TableState},
    Terminal,
};

const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

fn setup_tracing() {
    // setup tracing with some defaults if nothing is set
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

fn setup_terminal() {
    let mut stdout = std::io::stdout();

    execute!(stdout, cursor::Hide).unwrap();
    execute!(stdout, terminal::EnterAlternateScreen).unwrap();

    execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();

    terminal::enable_raw_mode().unwrap();
}

fn cleanup_terminal() {
    let mut stdout = std::io::stdout();

    execute!(stdout, cursor::MoveTo(0, 0)).unwrap();
    execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();

    execute!(stdout, terminal::LeaveAlternateScreen).unwrap();
    execute!(stdout, cursor::Show).unwrap();

    terminal::disable_raw_mode().unwrap();
}

enum State {
    Browsing,
    Selected,
}

// TODO: start spliting this up once I know where I'm going with this.
struct App {
    state: State,
    servers: Vec<(insim::protocol::relay::HostInfo, String)>,
    servers_state: TableState,

    insim_connected: bool,
    chat: BoundedVecDeque<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: State::Browsing,
            servers: Vec::new(),
            servers_state: TableState::default(),

            insim_connected: false,

            chat: BoundedVecDeque::new(20),
        }
    }

    pub fn push_chat(&mut self, data: String) {
        self.chat.push_front(format!(
            "{}: {}",
            chrono::Local::now().format(DATETIME_FORMAT),
            data,
        ));
    }

    pub fn next_server(&mut self) {
        let i = match self.servers_state.selected() {
            Some(i) => {
                if i >= self.servers.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.servers_state.select(Some(i));
    }

    pub fn previous_server(&mut self) {
        let i = match self.servers_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.servers.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.servers_state.select(Some(i));
    }
}

#[tokio::main]
pub async fn main() {
    setup_tracing();

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();
    let mut events = EventStream::new();

    setup_terminal();

    let mut client = insim::client::Config::default()
        .relay()
        .try_reconnect(true)
        .try_reconnect_attempts(2000)
        .build();

    let mut app = App::new();

    loop {
        tokio::select! {

            Some(Ok(e)) = events.next() => match (e, &app.state) {
                (
                    Event::Key(KeyEvent{ code: KeyCode::Char('q') | KeyCode::Esc, .. }),
                    State::Browsing,
                ) => {
                    client.shutdown();
                    break;
                },

                (
                    Event::Key(KeyEvent{ code: KeyCode::Char('q') | KeyCode::Esc, .. }),
                    State::Selected,
                ) => {
                    app.state = State::Browsing;
                },

                (
                    Event::Key(KeyEvent{ code: KeyCode::Char('r'), .. }),
                    State::Browsing
                ) => {
                    app.servers.clear();

                    let _ = client.send(
                        insim::client::Event::Packet(
                            insim::protocol::relay::HostListRequest::default().into()
                        )
                    ).await;
                },

                (
                    Event::Key(KeyEvent{ code: KeyCode::Up, .. }),
                    State::Browsing
                ) => {
                    app.previous_server();
                },

                (
                    Event::Key(KeyEvent{ code: KeyCode::Down, .. }),
                    State::Browsing
                ) => {
                    app.next_server();
                },

                (
                    Event::Key(KeyEvent{ code: KeyCode::Enter, .. }),
                    State::Browsing
                ) => {
                    if let Some(selected) = app.servers_state.selected() {
                        let _ = client
                        .send(insim::client::Event::Packet(
                            insim::protocol::relay::HostSelect {
                                hname: app.servers[selected].0.hname.to_owned(),
                                ..Default::default()
                            }
                            .into(),
                        ))
                        .await;

                        app.state = State::Selected;
                        app.push_chat(format!("Selected {}", app.servers[selected].0.hname));
                    }
                },

                (Event::Resize(..), _) => {
                    let res = terminal.autoresize();
                    if res.is_err() {
                        tracing::error!("auto-resize failed: {:?}", res);
                        break;
                    }
                },

                _ => {}
            },

            Some(e) = client.next() => match e {
                insim::client::Event::Disconnected => {
                    app.servers.clear();
                    app.insim_connected = false;
                    app.state = State::Browsing;
                },
                insim::client::Event::Connected => {
                    app.servers.clear();
                    app.insim_connected = true;

                    app.push_chat("Connected".into());

                    let _ = client.send(
                        insim::client::Event::Packet(
                            insim::protocol::relay::HostListRequest::default().into()
                        )
                    ).await;
                },

                insim::client::Event::Packet(frame) => match frame {
                    insim::protocol::Packet::RelayHostList(insim::protocol::relay::HostList { hinfo, .. }) => {
                        for info in hinfo.iter() {
                            app.servers.push(
                                (
                                    info.to_owned(),
                                    insim::string::colours::strip(
                                        info.hname.to_lossy_string()
                                    )
                                )
                            );

                            if info.flags.contains(insim::protocol::relay::HostInfoFlags::LAST) {
                                app.servers.sort_by(|(a, _), (b, _)| b.numconns.partial_cmp(&a.numconns).unwrap());
                            }
                        }
                    },

                    insim::protocol::Packet::MessageOut(data) => {
                        app.push_chat(
                            insim::string::colours::strip(data.msg.to_string())
                        );
                    },

                    insim::protocol::Packet::Lap(data) => {
                       app.push_chat(format!(
                            "lap plid={} lap={}",
                            data.plid,
                            data.lapsdone,
                        ));
                    },

                    insim::protocol::Packet::SplitX(data) => {
                       app.push_chat(format!(
                            "split plid={} split={} etime={}",
                            data.plid,
                            data.split,
                            data.etime,
                        ));
                    },

                    _ => { continue; }
                }

                _ => { continue; }
            }

        };

        // draw
        let res = terminal.draw(|f| {
            let outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());

            let header = Block::default().borders(Borders::ALL).title("insim.rs");

            let lines = if app.insim_connected {
                Text::from("Connected ")
            } else {
                Text::from("Not Connected ")
            };

            let help = Paragraph::new(lines)
                .block(header)
                .alignment(Alignment::Right);
            f.render_widget(help, outer[0]);

            let inner = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(10)].as_ref())
                .split(outer[1]);

            // Chat
            let chat_items: Vec<ListItem> = app
                .chat
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
            f.render_widget(items, inner[1]);

            match app.state {
                State::Browsing => {
                    let header_cells = ["Server", "Track", "Connections"]
                        .iter()
                        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
                    let header = Row::new(header_cells).height(1);

                    let rows = app.servers.iter().map(|(hinfo, name)| {
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

                    f.render_stateful_widget(t, inner[0], &mut app.servers_state);
                }

                State::Selected => {
                    let t = Block::default().title("Players");
                    f.render_widget(t, inner[0]);
                }
            }
        });

        if res.is_err() {
            tracing::error!("failed to draw terminal");
            break;
        }
    }

    cleanup_terminal();
}
