mod cli;
mod config;
mod peer;

pub type Result<T, E = Report> = color_eyre::Result<T, E>;
// A generic error report
// Produced via `Err(some_err).wrap_err("Some context")`
// or `Err(color_eyre::eyre::Report::new(SomeError))`
pub struct Report(color_eyre::Report);

impl std::fmt::Debug for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> From<E> for Report
where
    E: Into<color_eyre::Report>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

use insim::{self, codec::Frame};
use cli::Cli;

use clap::Parser;
use tokio::time::sleep;
use std::{io::IsTerminal, process::ExitCode, time::Duration};

#[tokio::main]
async fn main() -> Result<ExitCode> {
    color_eyre::config::HookBuilder::default()
        .theme(if !std::io::stderr().is_terminal() {
            // Don't attempt color
            color_eyre::config::Theme::new()
        } else {
            color_eyre::config::Theme::dark()
        })
        .install()?;

    let cli = Cli::parse();
    cli.instrumentation.setup()?;

    tracing::info!("hello! infop");
    tracing::trace!("hello! treacing");
    tracing::debug!("hello! debug");

    let config = config::Config::from_file(&cli.config)?;

    for (name, peer) in config.peers.iter() {

        use config::peer::PeerConfig;
        use insim::v9::Packet;
        use insim::connection::{Connection, Event};
        use insim::error::Error;

        use peer::Peer;

        let client: Connection<Packet> = match peer {
            PeerConfig::Relay { auto_select_host, websocket, spectator, .. } => {

                Connection::relay(
                    auto_select_host.clone(),
                    *websocket,
                    spectator.clone(),
                    insim::v9::Packet::isi_default(),
                )

            },
            _ => {
                todo!()
            }
            
        };

        let peer = Peer::new(client);

    }

    tokio::signal::ctrl_c().await?;

    Ok(ExitCode::SUCCESS)
}
