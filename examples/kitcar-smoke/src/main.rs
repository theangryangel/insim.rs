//! End-to-end smoke test / getting-started example for `kitcar`.
//!
//! One [`Service`] demonstrating:
//! - Packets and world events as inputs (`Mso`, `Connected`, `WorldEvent::Ready`)
//! - Typed chat commands via `ChatParser<C>`
//! - A repeating keyed timer (the heartbeat)
//! - A view drawn straight from the service's state: nothing to keep in sync
//! - Self-contained components with per-player state: a click counter that
//!   reports each click to the service, and a help dialog opened by `!help`
//! - A service stopping itself (`!quit`), after which the host shuts down
//!
//! Run with:
//!     cargo run -p kitcar-smoke -- --addr 127.0.0.1:29999
//!     cargo run -p kitcar-smoke -- --addr 127.0.0.1:29999 --admin-password hunter2

#![allow(missing_docs)]

use std::time::Duration;

use clap::Parser;
use insim::{
    Session,
    identifiers::{ConnectionId, RequestId},
    insim::{BtnStyle, Isi},
};
use kitcar::{
    AppError, ChatParser, Context, Effects, Host, Input, Service, WorldEvent,
    ui::{self, Child, Component, Node, Ui},
    world::Connected,
};

// Typed chat commands. `ChatParser<C>` runs `C::from_str` on each chat message.
#[derive(Debug, Clone)]
enum Cmd {
    Hello,
    Help,
    Echo { message: String },
    Quit,
}

impl std::str::FromStr for Cmd {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ChatParser has already stripped the `!` prefix.
        let (head, rest) = s.split_once(char::is_whitespace).unwrap_or((s, ""));
        match head {
            "hello" => Ok(Cmd::Hello),
            "help" => Ok(Cmd::Help),
            "echo" if !rest.trim().is_empty() => Ok(Cmd::Echo {
                message: rest.trim().to_string(),
            }),
            "quit" => Ok(Cmd::Quit),
            _ => Err(()),
        }
    }
}

// A self-contained component: each player gets their own count, kept by the
// host between renders. It reports every click to its parent.
#[derive(Default)]
struct ClickCounter {
    clicks: u32,
}

#[derive(Clone, Debug)]
struct Clicked;

impl Component for ClickCounter {
    type Msg = Clicked;
    type Out = u32;
    type Props<'a> = ();

    fn update(&mut self, Clicked: Clicked) -> Option<u32> {
        self.clicks += 1;
        Some(self.clicks)
    }

    fn render(&self, (): (), _: &mut Ui<'_, Clicked>) -> Node<Clicked> {
        ui::col([
            ui::text(format!("Your clicks: {}", self.clicks), BtnStyle::default())
                .w(40)
                .h(5),
            ui::clickable("Click me", BtnStyle::default(), Clicked)
                .w(40)
                .h(5),
        ])
    }
}

// Another self-contained component: opened by the service with
// `Effects::send(ucid, "help", DialogMsg::Show)`, closed by its own button.
#[derive(Default)]
struct HelpDialog {
    open: bool,
}

#[derive(Clone, Debug)]
enum DialogMsg {
    Show,
    Close,
}

impl Component for HelpDialog {
    type Msg = DialogMsg;
    type Out = ();
    type Props<'a> = ();

    fn update(&mut self, msg: DialogMsg) -> Option<()> {
        self.open = matches!(msg, DialogMsg::Show);
        None
    }

    fn render(&self, (): (), _: &mut Ui<'_, DialogMsg>) -> Node<DialogMsg> {
        if !self.open {
            return ui::empty();
        }
        ui::col([
            ui::text("^3Commands", BtnStyle::default().dark())
                .w(50)
                .h(6),
            ui::text("!hello  !echo <text>  !quit", BtnStyle::default().dark())
                .w(50)
                .h(5),
            ui::clickable("Close", BtnStyle::default().dark(), DialogMsg::Close)
                .w(50)
                .h(5),
        ])
        .position_absolute()
        .top(60)
        .left(75)
    }
}

#[derive(Debug, PartialEq)]
struct Heartbeat;

#[derive(Debug, Clone)]
enum Msg {
    Beat,
    Clicked { ucid: ConnectionId, total: u32 },
}

struct Smoke {
    chat: ChatParser<Cmd>,
    beats: u32,
    clicks: u32,
}

impl Smoke {
    fn command(&mut self, cmd: Cmd, ucid: ConnectionId) -> Effects<Msg> {
        let all = ConnectionId::ALL;
        match cmd {
            Cmd::Hello => Effects::new().message("hi from the smoke service!", all),
            Cmd::Help => Effects::new().send(ucid, "help", DialogMsg::Show),
            Cmd::Echo { message } => Effects::new().message(format!("echo: {message}"), all),
            Cmd::Quit => Effects::new().stop(),
        }
    }
}

impl Service for Smoke {
    type Msg = Msg;

    fn update(&mut self, input: Input<Msg>, cx: &Context<'_>) -> Effects<Msg> {
        match input {
            Input::Started => Effects::new().every(Heartbeat, Duration::from_secs(30), Msg::Beat),
            Input::World(WorldEvent::Ready(_)) => {
                tracing::info!(online = cx.world().connection_count(), "world ready");
                Effects::none()
            },
            // Only greet people who join now, not those already connected.
            Input::World(WorldEvent::Connected(Connected {
                info,
                existing: false,
            })) => {
                let online = cx.world().connection_count();
                Effects::new().message(
                    format!(
                        "^2Welcome ^7{} ^8(now {online} online) - try !help",
                        info.uname
                    ),
                    info.ucid,
                )
            },
            Input::Packet(packet) => match self.chat.parse(&packet) {
                Some(event) => self.command(event.parsed, event.ucid),
                None => Effects::none(),
            },
            Input::Msg(Msg::Beat) => {
                self.beats += 1;
                Effects::new().message(format!("[heartbeat] {}", self.beats), ConnectionId::ALL)
            },
            Input::Msg(Msg::Clicked { ucid, total }) => {
                self.clicks += 1;
                tracing::info!(%ucid, total, everyone = self.clicks, "clicked");
                Effects::none()
            },
            Input::Stopping => Effects::new().message("shutting down...", ConnectionId::ALL),
            _ => Effects::none(),
        }
    }

    // Drawn from the service's own state on every change.
    fn view(
        &self,
        ucid: ConnectionId,
        ui: &mut Ui<'_, Msg>,
        cx: &Context<'_>,
    ) -> Option<Node<Msg>> {
        let status = ui::col([
            ui::text(
                format!("Online: {}", cx.world().connection_count()),
                BtnStyle::default(),
            )
            .w(40)
            .h(5),
            ui::text(
                format!("Everyone's clicks: {}", self.clicks),
                BtnStyle::default(),
            )
            .w(40)
            .h(5),
            ui.child(
                Child::new(ClickCounter::default).on(move |total| Msg::Clicked { ucid, total }),
            ),
        ]);
        Some(
            ui::row([
                status,
                ui::text(
                    format!("[STATUS] beats: {}", self.beats),
                    BtnStyle::default(),
                )
                .w(40)
                .h(5),
                ui.child(Child::new(HelpDialog::default).key("help")),
            ])
            .w(200)
            .justify_between(),
        )
    }
}

#[derive(Parser, Debug)]
#[command(about = "kitcar smoke test / getting-started example")]
struct Args {
    /// LFS InSim address (host:port).
    #[arg(long, default_value = "127.0.0.1:29999")]
    addr: String,

    /// InSim admin password, if the host requires one.
    #[arg(long)]
    admin_password: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let stream = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::net::TcpStream::connect(args.addr.as_str()),
    )
    .await
    .map_err(insim::Error::from)?
    .map_err(insim::Error::from)?;
    stream.set_nodelay(true).map_err(insim::Error::from)?;
    let isi = Isi {
        reqi: RequestId(1),
        iname: "kitcar".into(),
        prefix: '!',
        admin: args.admin_password.unwrap_or_default(),
        ..Isi::default()
    };
    let session = Session::tcp(stream)?.handshake(isi).await?;

    let mut host = Host::spawn(session);
    let mut service = host
        .start(Smoke {
            chat: ChatParser::new(&['!']),
            beats: 0,
            clicks: 0,
        })
        .await?;

    tokio::select! {
        // `!quit` stops the service; then close the connection politely.
        () = service.finished() => host.shutdown().await,
        result = host.closed() => result,
    }
}
