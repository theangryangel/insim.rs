//! Bomb: an in-memory fork of the anomalous mini-game, for testing and
//! development. No database: the leaderboard lives for as long as the process.
//!
//! Drive through InSim circles in order before your timer runs out. Each one
//! shrinks the window; a finish object resets it; resets, pitting and collisions
//! cost you. Type `!help` or `!about` in chat, or click `?`.
//!
//! The layout needs InSim circles (and optionally InSim checkpoints as finish
//! objects). Load one by name from the server, or send one from a `.lyt` file:
//!
//!     cargo run -p bomb -- --addr 127.0.0.1:29999 --track FE1X --layout bomb
//!     cargo run -p bomb -- --addr 127.0.0.1:29999 --layout-file ./bomb.lyt

#![allow(missing_docs)]

mod game;
mod rules;
mod view;

use std::{path::PathBuf, time::Duration};

use clap::Parser;
use insim::{
    Session,
    core::track::Track,
    identifiers::RequestId,
    insim::{Isi, IsiFlags},
};
use insim_extra::kitcar::{Host, Layout, Setup};

use crate::{game::Bomb, rules::Config};

#[derive(Parser, Debug)]
#[command(about = "Bomb mini-game, in memory")]
struct Args {
    /// LFS InSim address (host:port).
    #[arg(long, default_value = "127.0.0.1:29999")]
    addr: String,

    /// InSim admin password, if the host requires one.
    #[arg(long)]
    admin_password: Option<String>,

    /// LFS track code (e.g. BL1, AS1, FE1X).
    #[arg(long, default_value = "FE1X")]
    track: Track,

    /// A layout saved on the server, loaded with /axload.
    #[arg(long, conflicts_with = "layout_file")]
    layout: Option<String>,

    /// A .lyt file to send to the server.
    #[arg(long, value_name = "PATH")]
    layout_file: Option<PathBuf>,

    /// Seconds to reach the first checkpoint, and after each finish.
    #[arg(long, default_value_t = 30)]
    timeout_secs: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    let args = Args::parse();

    let layout = match (args.layout, args.layout_file) {
        (Some(name), _) => Some(Layout::Named(name)),
        (None, Some(path)) => Some(Layout::Objects(insim_lyt::Lyt::from_path(&path)?.objects)),
        (None, None) => None,
    };
    let config = Config {
        checkpoint_timeout: Duration::from_secs(args.timeout_secs),
        ..Config::default()
    };

    let stream = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::net::TcpStream::connect(args.addr.as_str()),
    )
    .await??;
    stream.set_nodelay(true)?;
    let isi = Isi {
        reqi: RequestId(1),
        iname: "bomb".into(),
        prefix: '!',
        admin: args.admin_password.unwrap_or_default(),
        // Layout objects (to learn the circle order) and car contacts.
        flags: IsiFlags::AXM_LOAD | IsiFlags::CON,
        ..Isi::default()
    };
    let session = Session::tcp(stream)?.handshake(isi).await?;

    let mut host = Host::spawn(session);
    let setup = Setup::untimed(args.track).layout(layout);
    let mut bomb = host.start(Bomb::new(config, setup)).await?;

    tokio::select! {
        () = bomb.finished() => host.shutdown().await?,
        result = host.closed() => result?,
        _ = tokio::signal::ctrl_c() => host.shutdown().await?,
    }
    Ok(())
}
