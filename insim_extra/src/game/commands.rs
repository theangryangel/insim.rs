//! Parameter types for game command methods on [`crate::world::World`].

/// Grid access mode for [`World::change_grid`](crate::world::World::change_grid).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridMode {
    /// Non-admins can freely join, leave, and move others on the grid.
    Open,
    /// Non-admins can only move themselves on the grid.
    Slf,
    /// Only admins can modify the grid.
    Lock,
}

impl std::fmt::Display for GridMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            GridMode::Open => "open",
            GridMode::Slf => "self",
            GridMode::Lock => "lock",
        };
        write!(f, "{s}")
    }
}

/// Month of the year, used in [`TimeSet`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Month {
    /// January
    Jan,
    /// February
    Feb,
    /// March
    Mar,
    /// April
    Apr,
    /// May
    May,
    /// June
    Jun,
    /// July
    Jul,
    /// August
    Aug,
    /// September
    Sep,
    /// October
    Oct,
    /// November
    Nov,
    /// December
    Dec,
}

impl std::fmt::Display for Month {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Month::Jan => "Jan",
            Month::Feb => "Feb",
            Month::Mar => "Mar",
            Month::Apr => "Apr",
            Month::May => "May",
            Month::Jun => "Jun",
            Month::Jul => "Jul",
            Month::Aug => "Aug",
            Month::Sep => "Sep",
            Month::Oct => "Oct",
            Month::Nov => "Nov",
            Month::Dec => "Dec",
        };
        write!(f, "{s}")
    }
}

/// Parameters for [`World::time_set`](crate::world::World::time_set).
///
/// All fields are optional; only the parts that are `Some` are included in the
/// `/time set` command string.
///
/// # Example
/// ```rust,ignore
/// world.time_set(TimeSet {
///     date: Some((23, Month::Jan)),
///     time: Some((16, 0)),
///     utc_offset: Some(5),
/// })
/// ```
#[derive(Debug, Default, Clone)]
pub struct TimeSet {
    /// Day and month `(1..=31, Month)`. Both must be provided together.
    pub date: Option<(u8, Month)>,
    /// Hour and minute `(0..=23, 0..=59)`.
    pub time: Option<(u8, u8)>,
    /// UTC offset in whole hours (e.g. `5` -> `utc+5`, `-3` -> `utc-3`).
    pub utc_offset: Option<i8>,
}

/// Preset time-of-day for [`World::time_demo`](crate::world::World::time_demo).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeDemoPreset {
    /// Morning preset.
    Morning,
    /// Afternoon preset.
    Afternoon,
    /// Sunset preset.
    Sunset,
}

impl std::fmt::Display for TimeDemoPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TimeDemoPreset::Morning => "morning",
            TimeDemoPreset::Afternoon => "afternoon",
            TimeDemoPreset::Sunset => "sunset",
        };
        write!(f, "{s}")
    }
}
