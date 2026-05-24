use std::time::Duration;

use clap::Parser;
use insim::core::track::Track;

pub(super) const MIN_PLAYERS: usize = 1;

#[derive(Parser, Debug)]
pub struct MetronomeArgs {
    #[command(flatten)]
    pub insim: crate::args::InSimArgs,

    /// Target duration in milliseconds that players should aim to match.
    #[arg(long, default_value = "30000")]
    pub target_ms: u64,

    /// LFS track code (e.g. BL1, AS1, SO1).
    #[arg(long, default_value = "FE1X")]
    pub track: Track,

    /// Optional autocross layout name (loaded via /axload).
    #[arg(long)]
    pub layout: Option<String>,
}

/// Full runtime configuration for the metronome game, including optional DB backing.
pub struct MetronomeRunConfig {
    pub insim: crate::args::InSimArgs,
    pub config: MetronomeConfig,
    pub db: Option<(crate::db::Pool, i64)>,
}

#[derive(Clone, Debug)]
pub struct MetronomeConfig {
    pub target: Duration,
    pub track: Track,
    pub layout: Option<String>,
    pub setup_timeout: Duration,
}
