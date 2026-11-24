//! The server's time of day, as host commands.
//!
//! LFS has no packet for the time of day; it's set by typing `/time` commands.
//! A [`TimeOfDay`] turns into those commands with
//! [`to_commands`](TimeOfDay::to_commands); send each in an
//! [`Mst`](insim::insim::Mst).
//!
//! The server keeps its time settings across track changes, so a
//! [`TimeOfDay`] always states everything its mode uses: set mode always sends
//! a speed, and live mode always sends an offset. One track's frozen clock
//! never leaks into the next.

use std::fmt;

/// How the server's clock runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfDay {
    /// The real time at the track (`/time live`), shifted by `offset`.
    Live {
        /// Shift from the real time. [`TimeOffset::NONE`] for none.
        offset: TimeOffset,
    },
    /// A clock set by hand (`/time set`), running at `speed`.
    Set {
        /// Day and month. `None` keeps the server's date.
        date: Option<(u8, Month)>,
        /// Hour (0 to 23) and minute (0 to 59). `None` keeps the server's
        /// time.
        at: Option<(u8, u8)>,
        /// Whole hours from UTC. `None` uses the track's own time zone.
        utc: Option<i8>,
        /// How fast time runs (`/time mul`): 0 stops the clock, 1 is real
        /// time, higher is faster.
        speed: u8,
    },
    /// One of LFS's preset times (`/time demo`). It needs set mode, so
    /// `/time set` goes first.
    Demo(TimeDemoPreset),
}

impl TimeOfDay {
    /// The real time at the track, unshifted.
    pub fn live() -> Self {
        Self::Live {
            offset: TimeOffset::NONE,
        }
    }

    /// The real time at the track, shifted by `offset`.
    pub fn live_offset(offset: TimeOffset) -> Self {
        Self::Live { offset }
    }

    /// Always `hour`:`minute`: the clock is stopped.
    pub fn fixed(hour: u8, minute: u8) -> Self {
        Self::starting_at(hour, minute, 0)
    }

    /// Starts at `hour`:`minute` and runs at `speed` (1 for real time).
    pub fn starting_at(hour: u8, minute: u8, speed: u8) -> Self {
        Self::Set {
            date: None,
            at: Some((hour, minute)),
            utc: None,
            speed,
        }
    }

    /// The host commands, in order: `["/time set 17:30", "/time mul 0"]`.
    pub fn to_commands(&self) -> Vec<String> {
        match self {
            Self::Live { offset } => vec!["/time live".to_string(), offset.to_command()],
            Self::Set {
                date,
                at,
                utc,
                speed,
            } => {
                let mut set = String::from("/time set");
                if let Some((day, month)) = date {
                    set.push_str(&format!(" {day} {month}"));
                }
                if let Some((hour, minute)) = at {
                    set.push_str(&format!(" {hour:02}:{minute:02}"));
                }
                if let Some(utc) = utc {
                    let sign = if *utc < 0 { '-' } else { '+' };
                    set.push_str(&format!(" utc{sign}{}", utc.unsigned_abs()));
                }
                vec![set, format!("/time mul {speed}")]
            },
            Self::Demo(preset) => vec!["/time set".to_string(), format!("/time demo {preset}")],
        }
    }
}

/// A shift from the real time, for [`TimeOfDay::Live`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimeOffset {
    /// Whole days, either way.
    pub days: i32,
    /// Minutes, either way.
    pub minutes: i32,
}

impl TimeOffset {
    /// No shift.
    pub const NONE: Self = Self {
        days: 0,
        minutes: 0,
    };

    /// `hours` either way.
    pub fn hours(hours: i32) -> Self {
        Self {
            days: 0,
            minutes: hours * 60,
        }
    }

    /// `days` either way.
    pub fn days(days: i32) -> Self {
        Self { days, minutes: 0 }
    }

    /// `/time offset +D +H:MM`. Both parts are always sent: alone, a bare
    /// `+4` reads as days.
    pub fn to_command(&self) -> String {
        let sign = |n: i32| if n < 0 { '-' } else { '+' };
        let minutes = self.minutes.unsigned_abs();
        format!(
            "/time offset {}{} {}{}:{:02}",
            sign(self.days),
            self.days.unsigned_abs(),
            sign(self.minutes),
            minutes / 60,
            minutes % 60
        )
    }
}

/// LFS's preset times, for [`TimeOfDay::Demo`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeDemoPreset {
    /// Morning.
    Morning,
    /// Afternoon.
    Afternoon,
    /// Sunset.
    Sunset,
}

impl fmt::Display for TimeDemoPreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Morning => "morning",
            Self::Afternoon => "afternoon",
            Self::Sunset => "sunset",
        })
    }
}

/// A month, for [`TimeOfDay::Set`]'s date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Month {
    Jan,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `Debug` already spells each as LFS wants it.
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_time_always_states_its_offset() {
        assert_eq!(
            TimeOfDay::live().to_commands(),
            ["/time live", "/time offset +0 +0:00"]
        );
        assert_eq!(
            TimeOfDay::live_offset(TimeOffset::hours(6)).to_commands(),
            ["/time live", "/time offset +0 +6:00"]
        );
        let back = TimeOffset {
            days: -1,
            minutes: -90,
        };
        assert_eq!(back.to_command(), "/time offset -1 -1:30");
    }

    #[test]
    fn set_time_always_states_its_speed() {
        assert_eq!(
            TimeOfDay::fixed(17, 30).to_commands(),
            ["/time set 17:30", "/time mul 0"]
        );
        assert_eq!(
            TimeOfDay::starting_at(6, 5, 60).to_commands(),
            ["/time set 06:05", "/time mul 60"]
        );
        let everything = TimeOfDay::Set {
            date: Some((23, Month::Jan)),
            at: Some((16, 0)),
            utc: Some(5),
            speed: 1,
        };
        assert_eq!(
            everything.to_commands(),
            ["/time set 23 Jan 16:00 utc+5", "/time mul 1"]
        );
        let west = TimeOfDay::Set {
            date: None,
            at: None,
            utc: Some(-3),
            speed: 1,
        };
        assert_eq!(west.to_commands()[0], "/time set utc-3");
    }
}
