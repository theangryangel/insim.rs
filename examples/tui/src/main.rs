extern crate insim;

use std::panic;

use futures::{SinkExt, StreamExt};
use tracing_subscriber;

mod style;
mod view;

use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode, KeyEvent},
    execute, terminal,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::Paragraph,
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

    panic::set_hook(Box::new(move |x| {
        cleanup_terminal();
        print!("{:?}", x);
    }));

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();
    let mut events = EventStream::new();

    setup_terminal();

    let mut client = insim::client::Config::default()
        .relay()
        .try_reconnect(true)
        .try_reconnect_attempts(2000)
        .build();

    let mut app = view::View::new();

    loop {
        tokio::select! {

            Some(Ok(e)) = events.next() => {
                match (e, &app.state) {
                    (
                        Event::Key(KeyEvent{ code: KeyCode::Char('q') | KeyCode::Esc, .. }),
                        view::ViewState::Browsing,
                    ) => {
                        client.shutdown();
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

                        let _ = client.send(
                            insim::client::Event::Packet(
                                insim::protocol::relay::HostListRequest::default().into()
                            )
                        ).await;
                    },

                    (
                        Event::Key(KeyEvent{ code: KeyCode::Up, .. }),
                        view::ViewState::Browsing
                    ) => {
                        app.previous_server();
                    },

                    (
                        Event::Key(KeyEvent{ code: KeyCode::Down, .. }),
                        view::ViewState::Browsing
                    ) => {
                        app.next_server();
                    },

                    (
                        Event::Key(KeyEvent{ code: KeyCode::Enter, .. }),
                        view::ViewState::Browsing
                    ) => {
                        if let Some(selected) = app.servers.selected() {
                            let _ = client
                            .send(insim::client::Event::Packet(
                                insim::protocol::relay::HostSelect {
                                    hname: selected.clone(),
                                    ..Default::default()
                                }
                                .into(),
                            ))
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
                    insim::client::Event::Connected => {
                        let _ = client.send(
                            insim::client::Event::Packet(
                                insim::protocol::relay::HostListRequest::default().into()
                            )
                        ).await;
                    },

                    _ => {},
                }
            }
        };

        // draw
        let res = terminal.draw(|f| {
            let outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
                .split(f.size());

            let lines = if client.is_connected() {
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
            f.render_widget(state, outer[0]);

            app.render(f, outer[1]);
        });

        if res.is_err() {
            tracing::error!("failed to draw terminal");
            break;
        }
    }

    cleanup_terminal();
}
