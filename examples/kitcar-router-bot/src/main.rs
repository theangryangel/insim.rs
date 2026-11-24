//! A chat bot built on kitcar's `Router`: no `update` function, just handlers
//! for what it cares about.
//!
//! - Chat commands from `#[derive(ChatCommand)]`, including an admin-only one.
//!   Bad arguments get a usage reply for free.
//! - World events (`Connected`) and lifecycle markers (`Started`).
//! - Its own messages: a repeating timer and button clicks.
//! - A view drawn from the bot's state.
//! - `PenaltyClearer`, a ready-made service, running alongside.
//!
//! Run with:
//!     cargo run -p kitcar-router-bot -- --addr 127.0.0.1:29999
//!     cargo run -p kitcar-router-bot -- --addr 127.0.0.1:29999 --admin-password hunter2

#![allow(missing_docs)]

use std::time::Duration;

use clap::Parser;
use insim::{
    Session,
    identifiers::{ConnectionId, RequestId},
    insim::{Isi, Mst},
};
use insim_extra::{
    chat::ChatCommand,
    kitcar::{
        Context, Effects, Host,
        error::HostError,
        input::Started,
        services::{
            penalty_clearer::PenaltyClearer,
            router::{Caller, Router},
        },
    },
    ui::{self, Node, Ui},
    world::event::Connected,
};

/// Typed `!` commands. Doc comments are the help text.
#[derive(Debug, ChatCommand)]
enum Cmd {
    /// List the commands.
    Help,
    /// Say hello.
    Hello,
    /// Repeat a message to everyone.
    Echo { message: String },
    /// How many people the bot has greeted.
    Stats,
    /// Kick a player by username.
    #[chat(admin)]
    Kick { uname: String },
}

#[derive(Debug, Clone)]
enum Msg {
    /// The announcement is due.
    Announce,
    /// A player clicked the bot's button.
    Waved(ConnectionId),
}

#[derive(Debug, PartialEq)]
struct Announcement;

#[derive(Debug, Default)]
struct Bot {
    greeted: u32,
    waves: u32,
}

// Handlers are plain functions, picked by the type of their second argument.
fn started(_: &mut Bot, _: &Started, _: &Context<'_>) -> Effects<Msg> {
    Effects::new().every(Announcement, Duration::from_secs(300), Msg::Announce)
}

fn joined(bot: &mut Bot, event: &Connected, _: &Context<'_>) -> Option<Effects<Msg>> {
    // Greet only newcomers, not those already connected.
    if event.existing {
        return None;
    }
    bot.greeted += 1;
    Some(Effects::new().message(
        format!("^2Welcome ^7{}^2! Try !help", event.info.pname),
        event.info.ucid,
    ))
}

fn message(bot: &mut Bot, msg: Msg, cx: &Context<'_>) -> Effects<Msg> {
    match msg {
        Msg::Announce => Effects::new().message(
            format!("^3{} online - type !help", cx.world().connection_count()),
            ConnectionId::ALL,
        ),
        Msg::Waved(ucid) => {
            bot.waves += 1;
            let name = cx
                .world()
                .connection(ucid)
                .map_or("someone", |c| c.pname.as_str());
            Effects::new().message(format!("^7{name} ^3waved!"), ConnectionId::ALL)
        },
    }
}

fn command(bot: &mut Bot, caller: Caller, cmd: Cmd, _: &Context<'_>) -> Effects<Msg> {
    match cmd {
        Cmd::Help => {
            let lines: Vec<String> = Cmd::COMMANDS
                .iter()
                .filter(|c| caller.admin || !c.admin())
                .map(|c| format!("^7!{c} ^8- {}", c.help()))
                .collect();
            caller.reply(lines.join("\n"))
        },
        Cmd::Hello => caller.reply(format!("hi {}!", caller.uname)),
        Cmd::Echo { message } => Effects::new().message(message, ConnectionId::ALL),
        Cmd::Stats => caller.reply(format!(
            "greeted {}, waved at {} times",
            bot.greeted, bot.waves
        )),
        Cmd::Kick { uname } => Effects::new().packet(Mst::new(format!("/kick {uname}"))),
    }
}

fn view(bot: &Bot, ucid: ConnectionId, _: &mut Ui<'_, Msg>, _: &Context<'_>) -> Option<Node<Msg>> {
    Some(
        ui::clickable(
            format!("Wave ^8({})", bot.waves),
            ui::components::style::panel(),
            Msg::Waved(ucid),
        )
        .w(20)
        .h(5),
    )
}

#[derive(Parser, Debug)]
#[command(about = "A chat bot built with kitcar's Router")]
struct Args {
    /// LFS InSim address (host:port).
    #[arg(long, default_value = "127.0.0.1:29999")]
    addr: String,

    /// InSim admin password, if the host requires one.
    #[arg(long)]
    admin_password: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), HostError> {
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
        iname: "router-bot".into(),
        prefix: '!',
        admin: args.admin_password.unwrap_or_default(),
        ..Isi::default()
    };
    let session = Session::tcp(stream)?.handshake(isi).await?;

    let mut host = Host::spawn(session);
    let bot = Router::<_, Msg>::with_messages(Bot::default())
        .on(started)
        .on(joined)
        .on_msg(message)
        .commands(command)
        .view(view);
    let _bot = host.start(bot).await?;
    let _clearer = host
        .start(PenaltyClearer::new(Duration::from_secs(15)))
        .await?;

    host.closed().await
}
