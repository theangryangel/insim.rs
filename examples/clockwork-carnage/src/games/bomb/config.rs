use std::time::Duration;

use clap::Parser;
use insim::core::track::Track;

pub(super) const MIN_PLAYERS: usize = 2;
pub(super) const TICK_PERIOD: Duration = Duration::from_millis(500);
pub(super) const COLLISION_THRESHOLD_MPS: f32 = 30.0;
pub(super) const PENALTY_CLEAR_DELAY: Duration = Duration::from_secs(15);

#[derive(Parser, Debug)]
pub struct BombArgs {
    #[command(flatten)]
    pub insim: crate::args::InSimArgs,

    /// LFS track code (e.g. BL1, AS1, SO1).
    #[arg(long, default_value = "FE1X")]
    pub track: Track,

    /// Optional autocross layout name (loaded via /axload).
    #[arg(long)]
    pub layout: Option<String>,
}

/// Full runtime configuration for the bomb game, including optional DB backing.
pub struct BombRunConfig {
    pub insim: crate::args::InSimArgs,
    pub config: BombConfig,
    pub db: Option<(crate::db::Pool, i64)>,
}

#[derive(Clone, Debug)]
pub struct BombConfig {
    pub checkpoint_timeout: Duration,
    pub checkpoint_penalty: Duration,
    pub collision_max_penalty: Duration,
    pub track: Track,
    pub layout: Option<String>,
    pub setup_timeout: Duration,
}

impl Default for BombConfig {
    fn default() -> Self {
        Self {
            checkpoint_timeout: Duration::from_secs(30),
            checkpoint_penalty: Duration::from_millis(250),
            collision_max_penalty: Duration::from_millis(500),
            track: Track::default(),
            layout: None,
            setup_timeout: Duration::from_secs(60),
        }
    }
}
