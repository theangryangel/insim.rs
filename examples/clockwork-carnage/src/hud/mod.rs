mod dialog;
mod marquee;
mod scoreboard;
pub mod theme;
mod topbar;

pub use dialog::{Dialog, DialogMsg, DialogProps};
pub use marquee::{Marquee, MarqueeProps};
pub use scoreboard::{BombLeaderboard, ChallengeLeaderboard, MetronomeLeaderboard, bomb_scoreboard, challenge_scoreboard, metronome_scoreboard};
pub use topbar::topbar;

/// Format a `Duration` as `m:ss.xxx`, matching the web UI's `format_time_ms` filter.
pub fn format_duration(d: std::time::Duration) -> String {
    let ms = d.as_millis() as u64;
    let minutes = ms / 60_000;
    let seconds = (ms % 60_000) / 1000;
    let millis = ms % 1000;
    format!("{minutes}:{seconds:02}.{millis:03}")
}
