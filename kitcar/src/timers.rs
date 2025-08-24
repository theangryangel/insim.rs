//! Timers
use std::{
    cell::{Cell, RefCell},
    time::{Duration, Instant},
};

/// Timer
#[derive(Debug)]
pub struct Timer {
    start_time: RefCell<Instant>,
    duration: Duration,
    finished: Cell<bool>,
    remaining: Cell<Option<u32>>,
}

impl Timer {
    fn new(duration: Duration, remaining: Option<u32>) -> Self {
        Self {
            start_time: RefCell::new(Instant::now()),
            duration,
            finished: Cell::new(false),
            remaining: Cell::new(remaining),
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
    /// Now only requires an immutable reference (&self).
    pub fn tick(&self) -> bool {
        if self.finished.get() {
            return false;
        }

        // We use .borrow() to get an immutable reference to the Instant.
        if Instant::now() >= *self.start_time.borrow() + self.duration {
            if let Some(r) = self.remaining.get() {
                if r <= 1 {
                    self.finished.set(true);
                } else {
                    self.remaining.set(Some(r - 1));
                }
            } else {
                // This is a repeating timer with no limit
            }
            // We use .borrow_mut() to get a mutable reference to reset the time.
            *self.start_time.borrow_mut() = Instant::now();
            true
        } else {
            false
        }
    }

    /// Is finished?
    pub fn is_finished(&self) -> bool {
        self.finished.get()
    }

    /// Does this reset?
    pub fn resets(&self) -> bool {
        self.remaining.get() != Some(1)
    }

    /// How many loops remaining?
    pub fn remaining_repeats(&self) -> Option<u32> {
        self.remaining.get()
    }

    /// Remaining duration
    /// Note: I've also corrected the logic here for you.
    pub fn remaining_duration(&self) -> Duration {
        if self.is_finished() {
            return Duration::ZERO;
        }

        // Calculate the next finish time and see how far away it is from now.
        let finish_time = *self.start_time.borrow() + self.duration;
        finish_time.saturating_duration_since(Instant::now())
    }
}
