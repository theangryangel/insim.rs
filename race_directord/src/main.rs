mod cli;
mod config;
mod connections;
mod web;

pub type InsimPacket = insim::v9::Packet;
pub type InsimConnection = insim::connection::Connection<InsimPacket>;
pub type InsimEvent = insim::connection::Event<InsimPacket>;
pub type InsimError = insim::error::Error;

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

use clap::Parser;
use cli::Cli;
use std::{io::IsTerminal, path::Path, process::ExitCode};

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

    let config = config::Config::try_parse(Path::new(&cli.config))?;
    let mut manager = connections::ConnectionManager::new();
    for (name, peer) in config.connections.iter() {
        manager.add_peer(name, peer.clone()).await?;
    }

    if_chain::if_chain! {
        if let Some(web_config) = config.web;
        if let Some(web_listen) = web_config.listen;
        then {
            web::run(&web_listen, manager.clone());
        }
    }

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Requesting shutdown");
                manager.shutdown().await?;
            },

            _ = manager.run() => {
                tracing::info!("All tasks shutdown");
                break;
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}
