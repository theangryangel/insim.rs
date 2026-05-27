#![allow(missing_docs, missing_debug_implementations)]

mod args;
mod components;
mod db;
mod event;
mod games;
mod web;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Clockwork Carnage")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the bomb mini-game ad-hoc (no DB).
    Bomb(games::bomb::BombArgs),
    /// Run the metronome mini-game ad-hoc (no DB).
    Metronome(games::metronome::MetronomeArgs),
    /// Run the shortcut mini-game ad-hoc (no DB).
    Shortcut(games::shortcut::ShortcutArgs),
    /// Run a DB-backed event (reads config from DB, writes results).
    Event(event::EventArgs),
    /// Serve the web dashboard.
    Web(web::WebArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Bomb(args) => games::bomb::run_bomb(args)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?,
        Commands::Metronome(args) => games::metronome::run_metronome(args)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?,
        Commands::Shortcut(args) => games::shortcut::run_shortcut(args)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?,
        Commands::Event(args) => event::run_event(args).await?,
        Commands::Web(args) => web::run_web(args).await?,
    }
    Ok(())
}
