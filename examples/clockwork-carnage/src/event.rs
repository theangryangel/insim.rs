//! `event` subcommand: reads game config from the database and runs the
//! appropriate game with DB result writes enabled.

use std::time::Duration;

use anyhow::Context as _;
use clap::Parser;

use crate::{
    db,
    games::{
        bomb::{BombConfig, BombRunConfig, run_bomb_with},
        metronome::{MetronomeConfig, MetronomeRunConfig, run_metronome_with},
        shortcut::{ShortcutConfig, ShortcutRunConfig, run_shortcut_with},
    },
};

#[derive(Parser, Debug)]
#[command(about = "Run a DB-backed event (reads config from DB, writes results)")]
pub struct EventArgs {
    /// Postgres connection string.
    #[arg(long, env = "DATABASE_URL")]
    pub db: String,

    /// Event ID to load config from and write results to.
    #[arg(long)]
    pub event_id: i64,

    #[command(flatten)]
    pub insim: crate::args::InSimArgs,
}

pub async fn run_event(args: EventArgs) -> anyhow::Result<()> {
    let pool = db::connect(&args.db)
        .await
        .context("failed to connect to database")?;

    let event = db::get_event(&pool, args.event_id)
        .await
        .context("failed to query event")?
        .with_context(|| format!("event {} not found", args.event_id))?;

    db::update_event_status(&pool, args.event_id, db::EventStatus::Live)
        .await
        .context("failed to set event status to live")?;

    let track = event.track;
    let layout = if event.layout.is_empty() {
        None
    } else {
        Some(event.layout.clone())
    };

    match &*event.mode {
        db::EventMode::Bomb {
            checkpoint_timeout_secs,
            checkpoint_penalty_ms,
            collision_max_penalty_ms,
        } => {
            let config = BombConfig {
                checkpoint_timeout: Duration::from_secs(*checkpoint_timeout_secs as u64),
                checkpoint_penalty: Duration::from_millis(*checkpoint_penalty_ms as u64),
                collision_max_penalty: Duration::from_millis(*collision_max_penalty_ms as u64),
                track,
                layout,
                setup_timeout: Duration::from_secs(60),
            };
            run_bomb_with(BombRunConfig {
                insim: args.insim,
                config,
                db: Some((pool, args.event_id)),
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        },

        db::EventMode::Metronome { target_ms } => {
            let config = MetronomeConfig {
                target: Duration::from_millis(*target_ms as u64),
                track,
                layout,
                setup_timeout: Duration::from_secs(60),
            };
            run_metronome_with(MetronomeRunConfig {
                insim: args.insim,
                config,
                db: Some((pool, args.event_id)),
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        },

        db::EventMode::Shortcut => {
            let config = ShortcutConfig {
                track,
                layout,
                setup_timeout: Duration::from_secs(60),
            };
            run_shortcut_with(ShortcutRunConfig {
                insim: args.insim,
                config,
                db: Some((pool, args.event_id)),
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        },
    }

    Ok(())
}
