//! Countdown

use std::time::Duration;

use tokio::time::Interval;

/// A countdown timer that ticks a specific number of times at a given interval.
///
/// Once the countdown reaches zero, subsequent calls to `tick()` will
/// immediately return `None`.
#[derive(Debug)]
pub struct Countdown {
    interval: Interval,
    remaining_ticks: u32,
}

impl Countdown {
    /// Creates a new `Countdown`
    pub fn new(period: Duration, ticks: u32) -> Self {
        let interval = tokio::time::interval(period);
        Self {
            interval,
            remaining_ticks: ticks,
        }
    }

    /// Waits for the next tick of the countdown.
    ///
    /// Returns `Some` with the number of ticks remaining *after* the current tick,
    /// or `None` if the countdown has completed.
    pub async fn tick(&mut self) -> Option<u32> {
        if self.remaining_ticks == 0 {
            return None;
        }

        let _ = self.interval.tick().await;
        self.remaining_ticks -= 1;
        Some(self.remaining_ticks)
    }

    /// The remaining Duration for the countdown.
    pub async fn remaining_duration(&self) -> Duration {
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
        // Use a tiny duration to make the test run fast.
        let mut countdown = Countdown::new(Duration::from_millis(1), total_ticks);

        // Expect the remaining ticks to count down from (total - 1) to 0
        assert_eq!(countdown.tick().await, Some(2));
        assert_eq!(countdown.tick().await, Some(1));
        assert_eq!(countdown.tick().await, Some(0));

        // After the final tick, it should return None
        assert_eq!(countdown.tick().await, None);

        // It should continue to return None on subsequent calls
        assert_eq!(countdown.tick().await, None);
    }

    #[tokio::test]
    async fn test_zero_tick_countdown_is_immediate() {
        // The duration doesn't matter here, as it should never be used.
        let mut countdown = Countdown::new(Duration::from_secs(10), 0);

        // The very first tick should return None.
        assert_eq!(countdown.tick().await, None);
    }
}
