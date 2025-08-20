//! Timers
use std::time::{Duration, Instant};

/// Timer
#[derive(Debug)]
pub struct Timer {
    start_time: Instant,
    duration: Duration,
    finished: bool,
    remaining: Option<u32>,
}

impl Timer {
    fn new(duration: Duration, remaining: Option<u32>) -> Self {
        Self {
            start_time: Instant::now(),
            duration,
            finished: false,
            remaining,
        }
    }

    /// Create a new one-shot timer
    pub fn once(duration: Duration) -> Self {
        Self::new(duration, Some(1))
    }

    /// Create a new repeating timer
    pub fn repeating(interval: Duration, remaining: Option<u32>) -> Self {
        Self::new(interval, remaining)
    }

    /// Tick
    pub fn tick(&mut self) -> bool {
        if self.finished {
            return false;
        }

        if Instant::now() >= self.start_time + self.duration {
            if let Some(r) = self.remaining {
                if r <= 1 {
                    self.finished = true;
                } else {
                    self.remaining = Some(r - 1);
                }
            } else {
                // This is a repeating timer with no limit
            }
            self.start_time = Instant::now();
            true
        } else {
            false
        }
    }

    /// Is finished?
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Does this reset?
    pub fn resets(&self) -> bool {
        self.remaining != Some(1)
    }

    /// How many loops remaining?
    pub fn remaining_repeats(&self) -> Option<u32> {
        self.remaining
    }
}
