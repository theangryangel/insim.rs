//! Time and timer related helpers.

use std::time::Duration;

use tokio::time::{Instant, Interval};

/// A countdown timer that ticks a specific number of times at a given interval.
///
/// Once the countdown reaches zero, subsequent calls to [`Countdown::tick`]
/// immediately return `None`.
///
/// ```ignore
/// use std::time::Duration;
/// use insim_app::time::Countdown;
///
/// let mut cd = Countdown::new(Duration::from_secs(1), 3);
/// while let Some(remaining) = cd.tick().await {
///     ui.modify(|g| g.countdown_text = format!("{remaining}..."));
/// }
/// // race starts here
/// ```
#[derive(Debug)]
pub struct Countdown {
    interval: Interval,
    remaining_ticks: u32,
}

impl Countdown {
    /// Construct a new countdown. The first tick fires after `period`.
    pub fn new(period: Duration, ticks: u32) -> Self {
        let interval = tokio::time::interval_at(Instant::now() + period, period);
        Self {
            interval,
            remaining_ticks: ticks,
        }
    }

    /// Wait for the next tick.
    ///
    /// Returns `Some` with the number of ticks remaining *after* the current
    /// tick, or `None` once the countdown has completed (and on all
    /// subsequent calls).
    pub async fn tick(&mut self) -> Option<u32> {
        if self.remaining_ticks == 0 {
            return None;
        }

        let _ = self.interval.tick().await;
        self.remaining_ticks -= 1;
        Some(self.remaining_ticks)
    }

    /// Total time remaining for the countdown across all unfired ticks.
    pub fn remaining_duration(&self) -> Duration {
        self.interval.period() * self.remaining_ticks
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_countdown_completes_correctly() {
        let total_ticks = 3;
        // Tiny duration keeps the test fast.
        let mut countdown = Countdown::new(Duration::from_millis(1), total_ticks);

        // Remaining count down from (total - 1) to 0.
        assert_eq!(countdown.tick().await, Some(2));
        assert_eq!(countdown.tick().await, Some(1));
        assert_eq!(countdown.tick().await, Some(0));

        // After the final tick, None.
        assert_eq!(countdown.tick().await, None);

        // Still None on subsequent calls.
        assert_eq!(countdown.tick().await, None);
    }

    #[tokio::test]
    async fn test_zero_tick_countdown_is_immediate() {
        // Duration doesn't matter - never used because ticks = 0.
        let mut countdown = Countdown::new(Duration::from_secs(10), 0);
        assert_eq!(countdown.tick().await, None);
    }
}
