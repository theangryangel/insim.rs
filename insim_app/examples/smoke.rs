//! End-to-end smoke test for the `insim_app` PoC.
//!
//! Wire-packet handlers, synthetic-event handlers, middleware emitting those
//! synthetic events, packet sending via the `Sender` extractor, and a
//! background ticker installed via `Event<Startup>` + `tokio::spawn`.
//!
//! Run with:
//!     cargo run -p insim_app --example smoke -- 127.0.0.1:29999
//!     cargo run -p insim_app --example smoke -- 127.0.0.1:29999 --admin-password hunter2

// Pulled in transitively by `insim_app`; silence unused-crate lint in this binary.
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use clap::Parser;
use fixedbitset as _;
use futures as _;
use insim::{
    identifiers::ConnectionId,
    insim::{BtnStyle, Mso, Ncn},
};
use insim_app::{
    App, AppError, ChatParser, Connected, Disconnected, Event, ExtractCx, FromContext, Handler,
    Packet, Presence, Sender, Startup, State, serve,
    ui::{self, Component, InvalidateHandle, Ui},
    util::mtc,
};
use taffy as _;
use thiserror as _;
use tokio_util as _;

#[derive(Clone)]
struct AppState {
    joins: Arc<AtomicUsize>,
}

async fn log_ncn(Packet(ncn): Packet<Ncn>, State(state): State<AppState>) -> Result<(), AppError> {
    let n = state.joins.fetch_add(1, Ordering::Relaxed) + 1;
    tracing::info!(ucid = %ncn.ucid, uname = %ncn.uname, total = n, "ncn");
    Ok(())
}

async fn welcome(
    Event(Connected(info)): Event<Connected>,
    presence: Presence,
    sender: Sender,
) -> Result<(), AppError> {
    let total = presence.count();
    sender.packet(mtc(
        format!("^2Welcome ^7{} ^8(now {total} online)", info.uname),
        Some(info.ucid),
    ))
}

async fn echo_mso(Packet(mso): Packet<Mso>) -> Result<(), AppError> {
    tracing::debug!(ucid = %mso.ucid, msg = %mso.msg, "mso");
    Ok(())
}

// ---------------------------------------------------------------------------
// Typed-enum chat: parser middleware + Event<C> handler.
//
// `ChatParser<C>` is a middleware that runs `C::from_str` exactly once per
// `Mso` and emits the parsed value as a synthetic event. Any number of
// `Event<C>` handlers can react to the result without re-parsing.
//
// Pairs naturally with `insim_extras::chat::Parse` - if you had that derive,
// you'd write:
//
//     #[derive(Debug, Clone, insim_extras::chat::Parse)]
//     #[chat(prefix = '!')]
//     enum Cmd { Hello, Echo { message: String } }
//
//     impl std::str::FromStr for Cmd {
//         type Err = insim_extras::chat::ParseError;
//         fn from_str(s: &str) -> Result<Self, Self::Err> {
//             <Self as insim_extras::chat::Parse>::parse(s)
//         }
//     }
//
// For this self-contained example we hand-roll a tiny FromStr below.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Cmd {
    Hello,
    Ping,
    Echo { message: String },
}

impl std::str::FromStr for Cmd {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let body = s.strip_prefix('!').ok_or(())?;
        let (head, rest) = body.split_once(char::is_whitespace).unwrap_or((body, ""));
        match head {
            "hello" => Ok(Cmd::Hello),
            "ping" => Ok(Cmd::Ping),
            "echo" if !rest.trim().is_empty() => Ok(Cmd::Echo {
                message: rest.trim().to_string(),
            }),
            _ => Err(()),
        }
    }
}

async fn handle_typed_chat(Event(cmd): Event<Cmd>, sender: Sender) -> Result<(), AppError> {
    match cmd {
        Cmd::Hello => {
            sender.packet(mtc("hi from typed handler!", Some(ConnectionId::ALL)))?;
        },
        Cmd::Ping => {
            sender.packet(mtc("pong", Some(ConnectionId::ALL)))?;
        },
        Cmd::Echo { message } => {
            sender.packet(mtc(
                format!("typed-echo: {message}"),
                Some(ConnectionId::ALL),
            ))?;
        },
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// A *non-function* handler: a named struct that owns its own state and
// implements `Handler<S, T>` directly. Useful when:
//   - you want a named, testable handler unit instead of an anonymous closure;
//   - the handler carries state that doesn't belong in the global `S` (e.g. a
//     local counter, a small LRU cache, a per-handler debouncer);
//   - you want to package extractor logic and behaviour together for reuse.
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct MsoCounter {
    seen: Arc<AtomicUsize>,
}

impl MsoCounter {
    fn new() -> Self {
        Self {
            seen: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl<S: Send + Sync + 'static> Handler<S, (Packet<Mso>,)> for MsoCounter {
    async fn call(self, cx: &ExtractCx<'_, S>) -> Result<(), AppError> {
        let Some(Packet(mso)) = <Packet<Mso> as FromContext<S>>::from_context(cx) else {
            return Ok(());
        };
        let n = self.seen.fetch_add(1, Ordering::Relaxed) + 1;
        tracing::info!(ucid = %mso.ucid, total = n, "MsoCounter saw chat");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Long-running background work: react to the one-shot `Startup` event and
// spawn a task. The task keeps a clone of `Sender` so it can send packets /
// emit events; when the runtime shuts down, the back-channel closes and the
// `send` call returns an error - the loop exits naturally.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// A tiny `ui` example: a 3-row in-game button panel per player.
//
//   row 1 - "Online: N"   (global state, refreshed when anyone joins/leaves)
//   row 2 - "Your clicks: N" (per-player state, owned by the SmokeView)
//   row 3 - clickable "Click me" button (emits SmokeUiMsg::ButtonClicked)
//
// The UI is registered once as an extension. It owns a background LocalSet
// thread that runs view tasks per connection. Handlers pull the `Ui` extractor
// to push global updates (`ui.assign(...)`); the SmokeView handles its own
// per-player state via `Component::update`.
// ---------------------------------------------------------------------------

#[derive(Clone, Default, Debug)]
struct UiGlobal {
    online: u64,
}

#[derive(Clone, Debug)]
enum SmokeUiMsg {
    ButtonClicked,
}

struct SmokeView {
    ucid: ConnectionId,
    clicks: u32,
}

impl Component for SmokeView {
    type Message = SmokeUiMsg;
    type Props<'a> = (&'a UiGlobal, &'a ());

    fn render(&self, (global, _): Self::Props<'_>) -> ui::Node<Self::Message> {
        ui::container()
            .flex()
            .flex_col()
            .with_child(
                ui::text(format!("Online: {}", global.online), BtnStyle::default())
                    .w(40.0)
                    .h(5.0),
            )
            .with_child(
                ui::text(format!("Your clicks: {}", self.clicks), BtnStyle::default())
                    .w(40.0)
                    .h(5.0),
            )
            .with_child(
                ui::clickable("Click me", BtnStyle::default(), SmokeUiMsg::ButtonClicked)
                    .w(40.0)
                    .h(5.0),
            )
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            SmokeUiMsg::ButtonClicked => {
                self.clicks += 1;
                tracing::info!(ucid = %self.ucid, clicks = self.clicks, "ui button clicked");
            },
        }
    }
}

async fn refresh_ui_count(
    presence: Presence,
    ui: Ui<SmokeView, UiGlobal, ()>,
) -> Result<(), AppError> {
    ui.assign(UiGlobal {
        online: presence.count() as u64,
    });
    Ok(())
}

async fn refresh_on_connect(
    _: Event<Connected>,
    presence: Presence,
    ui: Ui<SmokeView, UiGlobal, ()>,
) -> Result<(), AppError> {
    refresh_ui_count(presence, ui).await
}

async fn refresh_on_disconnect(
    _: Event<Disconnected>,
    presence: Presence,
    ui: Ui<SmokeView, UiGlobal, ()>,
) -> Result<(), AppError> {
    refresh_ui_count(presence, ui).await
}

/// Observe UI button clicks via the click-bridge: each click also flows
/// through the dispatcher as a synthetic `Event<SmokeUiMsg>`, so handlers
/// can react without going through `Component::update`.
async fn on_ui_click(Event(msg): Event<SmokeUiMsg>, sender: Sender) -> Result<(), AppError> {
    match msg {
        SmokeUiMsg::ButtonClicked => {
            sender.packet(mtc("button clicked!", Some(ConnectionId::ALL)))?;
        },
    }
    Ok(())
}

async fn install_ticker(_: Event<Startup>, sender: Sender) -> Result<(), AppError> {
    let _ticker = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        let _ = interval.tick().await; // skip immediate fire
        let mut beat: u32 = 0;
        loop {
            let _ = interval.tick().await;
            beat += 1;
            println!("[ticker] beat {beat}");
            if sender
                .packet(mtc(
                    format!("[ticker] beat {beat}"),
                    Some(ConnectionId::ALL),
                ))
                .is_err()
            {
                return;
            }
        }
    });
    Ok(())
}

#[derive(Parser, Debug)]
#[command(about = "insim_app PoC smoke test")]
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

    let app = App::<AppState>::new();
    let ui = Ui::<SmokeView, UiGlobal, ()>::new(
        app.sender().clone(),
        UiGlobal::default(),
        |ucid, _invalidator: InvalidateHandle| SmokeView { ucid, clicks: 0 },
    );

    let app = app
        .with_state(AppState {
            joins: Arc::new(AtomicUsize::new(0)),
        })
        .extension(Presence::new())
        .extension(ChatParser::<Cmd>::new())
        .extension(ui)
        .handler(install_ticker)
        .handler(log_ncn)
        .handler(welcome)
        .handler(refresh_on_connect)
        .handler(refresh_on_disconnect)
        .handler(on_ui_click)
        .handler(handle_typed_chat)
        .handler(echo_mso)
        .handler(MsoCounter::new());

    let builder = insim::tcp(args.addr)
        .isi_iname("insim_app".to_string())
        .isi_prefix('!')
        .isi_admin_password(args.admin_password);

    serve(builder, app).await
}
