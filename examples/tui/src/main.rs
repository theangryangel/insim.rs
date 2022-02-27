extern crate insim;

use std::panic;

use futures::StreamExt;
use tracing_subscriber;

mod style;
mod view;
mod widgets;

use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode, KeyEvent},
    execute, terminal,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

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

#[tokio::main]
pub async fn main() {
    setup_tracing();

    // ensure we recover the terminal on panic
    panic::set_hook(Box::new(move |x| {
        cleanup_terminal();
        print!("{:?}", x);
    }));

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();
    let mut events = EventStream::new();

    setup_terminal();

    let client = insim::client::Config::default()
        .relay()
        .try_reconnect(true)
        .try_reconnect_attempts(2000)
        .build();

    let mut app = view::View::new();

    loop {
        tokio::select! {

            // TODO: clean up
            Some(Ok(e)) = events.next() => {
                match (e, &app.state) {
                    (
                        Event::Key(KeyEvent{ code: KeyCode::Char('q') | KeyCode::Esc, .. }),
                        view::ViewState::Browsing,
                    ) => {
                        client.shutdown().await;
                        break;
                    },

                    (
                        Event::Key(KeyEvent{ code: KeyCode::Char('q') | KeyCode::Esc, .. }),
                        view::ViewState::Selected,
                    ) => {
                        app.state = view::ViewState::Browsing;
                    },

                    (
                        Event::Key(KeyEvent{ code: KeyCode::Char('r'), .. }),
                        view::ViewState::Browsing
                    ) => {
                        app.servers.clear();
                        app.players.clear();

                        let _ = client.send(
                            insim::protocol::relay::HostListRequest::default().into()
                        ).await;
                    },

                    (
                        Event::Key(KeyEvent{ code: KeyCode::Up, .. }),
                        view::ViewState::Browsing
                    ) => {
                        app.servers.scroll_prev();
                    },

                    (
                        Event::Key(KeyEvent{ code: KeyCode::Down, .. }),
                        view::ViewState::Browsing
                    ) => {
                        app.servers.scroll_next();
                    },

                    (
                        Event::Key(KeyEvent{ code: KeyCode::Up, .. }),
                        view::ViewState::Selected
                    ) => {
                        app.players.scroll_prev();
                    },

                    (
                        Event::Key(KeyEvent{ code: KeyCode::Down, .. }),
                        view::ViewState::Selected
                    ) => {
                        app.players.scroll_next();
                    },


                    (
                        Event::Key(KeyEvent{ code: KeyCode::Enter, .. }),
                        view::ViewState::Browsing
                    ) => {
                        if let Some(selected) = app.servers.selected() {
                            app.players.clear();

                            app.chat.push(format!("Selected to {}", selected));

                            let _ = client
                            .send(
                                insim::protocol::relay::HostSelect {
                                    hname: selected.clone(),
                                    ..Default::default()
                                }
                                .into(),
                            )
                            .await;

                            let _ = client
                            .send(
                                insim::protocol::insim::Tiny{
                                    reqi: 0,
                                    subtype: insim::protocol::insim::TinyType::Ncn,
                                }
                                .into(),
                            )
                            .await;

                            let _ = client
                            .send(
                                insim::protocol::insim::Tiny{
                                    reqi: 0,
                                    subtype: insim::protocol::insim::TinyType::Npl,
                                }
                                .into(),
                            )
                            .await;

                            app.state = view::ViewState::Selected;
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

                }
            },

            Some(e) = client.next() => {
                app.on_network(&e);

                match e {
                    insim::client::Event::State(insim::client::State::Connected) => {
                        let _ = client.send(
                            insim::protocol::relay::HostListRequest::default().into()
                        ).await;
                    },

                    _ => {},
                }
            }
        };

        // TODO: probably shouldn't draw on every event. if someone holds down a key we get high
        // cpu usage, duh.
        let res = terminal.draw(|f| {
            let outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
                .split(f.size());

            let connected = widgets::ConnectedWidget::default().connected(client.is_connected());
            f.render_widget(connected, outer[0]);

            let inner = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(10)].as_ref())
                .split(outer[1]);

            let chat = widgets::ChatWidget::default();
            f.render_stateful_widget(chat, inner[1], &mut app.chat);

            match app.state {
                view::ViewState::Browsing => {
                    let servers = widgets::ServersWidget::default();
                    f.render_stateful_widget(servers, inner[0], &mut app.servers);
                }

                view::ViewState::Selected => {
                    let players = widgets::PlayerListWidget::default();
                    f.render_stateful_widget(players, inner[0], &mut app.players);
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
