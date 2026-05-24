use std::time::Duration;

use clap::Parser;
use insim::core::track::Track;

pub(super) const MIN_PLAYERS: usize = 1;

#[derive(Parser, Debug)]
pub struct ShortcutArgs {
    #[command(flatten)]
    pub insim: crate::args::InSimArgs,

    /// LFS track code (e.g. BL1, AS1, SO1).
    #[arg(long, default_value = "FE1X")]
    pub track: Track,

    /// Optional autocross layout name (loaded via /axload).
    #[arg(long)]
    pub layout: Option<String>,
}

/// Full runtime configuration for the shortcut game, including optional DB backing.
pub struct ShortcutRunConfig {
    pub insim: crate::args::InSimArgs,
    pub config: ShortcutConfig,
    pub db: Option<(crate::db::Pool, i64)>,
}

#[derive(Clone, Debug)]
pub struct ShortcutConfig {
    pub track: Track,
    pub layout: Option<String>,
    pub setup_timeout: Duration,
}
