//! Game-state types used by [`crate::world::World`].
//!
//! The [`SessionKind`], [`MultiplayerState`], [`GameInfo`], and related types
//! are defined here. The actual state is owned by
//! [`World`](crate::world::World), which exposes query methods directly.

use std::time::Duration;

use insim::{
    core::{game_version::GameVersion, track::Track, vehicle::Vehicle, wind::Wind},
    insim::{PlcAllowedCarsSet, RaceFlags, RaceLaps, StaFlags},
};

/// The kind of session that an [`Rst`] packet started.
///
/// Carried by session-started events so consumers (notably the race
/// tracker) can interpret subsequent packets correctly - most importantly
/// `Fin`, which marks a finish in a race but fires after every lap in
/// qualifying.
#[derive(Debug, Clone, Copy)]
pub enum SessionKind {
    /// A race session (`Rst` with `qualmins == 0` and a lap/hour count).
    Race {
        /// Race length configuration.
        laps: RaceLaps,
        /// Race flags from the `Rst` packet.
        flags: RaceFlags,
    },
    /// A qualifying session (`Rst` with `qualmins > 0`).
    Qualifying {
        /// Qualifying duration.
        duration: Duration,
        /// Race flags from the `Rst` packet.
        flags: RaceFlags,
    },
    /// A practice session (`Rst` with `qualmins == 0` and
    /// [`RaceLaps::Practice`], i.e. 0 laps).
    ///
    /// LFS reports no race in progress, so this is only distinguishable from
    /// the `Rst` packet. Like qualifying, `Fin` here is a per-lap signal rather
    /// than a finish.
    Practice,
    /// An untimed open/cruise session (`Rst` with `qualmins == 0` and
    /// [`RaceLaps::Untimed`]).
    ///
    /// Like [`Practice`](Self::Practice) LFS reports no race in progress, and
    /// `Fin` is a per-lap signal rather than a finish.
    Untimed,
}

impl SessionKind {
    /// Whether this is a race session (DNF semantics apply, `Fin` marks a finish).
    pub fn is_race(self) -> bool {
        matches!(self, Self::Race { .. })
    }

    /// Whether this session kind warrants lap and entrant tracking.
    ///
    /// Returns `true` for [`Race`](Self::Race) and [`Qualifying`](Self::Qualifying).
    /// Returns `false` for [`Practice`](Self::Practice) and
    /// [`Untimed`](Self::Untimed) (sessions where race tracking is meaningless).
    pub fn is_tracking(self) -> bool {
        matches!(self, Self::Race { .. } | Self::Qualifying { .. })
    }
}

/// Whether LFS is currently running in local or multiplayer mode.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum MultiplayerState {
    /// LFS is in single-player / offline mode.
    #[default]
    Local,
    /// LFS is connected to a multiplayer session.
    Multiplayer {
        /// Host name of the server.
        host_name: String,
        /// `true` if this instance is the host; `false` if a guest.
        is_host: bool,
    },
}

/// Version information about the connected LFS instance, from a `Ver` packet.
#[derive(Debug, Default, Clone)]
pub struct VersionInfo {
    /// Product name (e.g. `"S3"`).
    pub product: String,
    /// LFS game version.
    pub version: GameVersion,
}

/// Snapshot of the game state, produced by [`World::game_info()`](crate::world::World::game_info).
#[derive(Debug, Default, Clone)]
pub struct GameInfo {
    /// Kind of the current session. `None` means lobby / no session active.
    pub(crate) session_kind: Option<SessionKind>,
    pub(crate) track: Option<Track>,
    pub(crate) layout: Option<String>,
    pub(crate) weather: Option<u8>,
    pub(crate) wind: Option<Wind>,
    pub(crate) flags: StaFlags,
    pub(crate) multiplayer: MultiplayerState,
    /// Server's allowed-cars set, from a `Small`/`Alc` reply. `None` until seen.
    pub(crate) allowed_cars: Option<PlcAllowedCarsSet>,
    /// Server's allowed-mods list, from a `Mal` packet. Empty = unrestricted or
    /// not yet received.
    pub(crate) allowed_mods: Vec<Vehicle>,
    /// Version information, from a `Ver` packet. `None` until received.
    pub(crate) version: Option<VersionInfo>,
    /// Incremented each time an `Axi` packet is applied, regardless of lname content.
    pub(crate) axi_count: u64,
    /// Incremented each time an `Rst` packet is applied.
    pub(crate) rst_count: u64,
}

impl GameInfo {
    /// Current session kind. `None` means lobby / no session active.
    pub fn session(&self) -> Option<SessionKind> {
        self.session_kind
    }

    /// Currently selected track, if known.
    pub fn current_track(&self) -> Option<&Track> {
        self.track.as_ref()
    }

    /// Currently loaded layout, if known.
    pub fn current_layout(&self) -> Option<&String> {
        self.layout.as_ref()
    }

    /// Weather identifier (0..=2 typically).
    pub fn weather(&self) -> Option<u8> {
        self.weather
    }

    /// Wind conditions.
    pub fn wind(&self) -> Option<&Wind> {
        self.wind.as_ref()
    }

    /// Overall game flags.
    pub fn flags(&self) -> &StaFlags {
        &self.flags
    }

    /// Current multiplayer state.
    ///
    /// [`MultiplayerState::Local`] until an `ISM` packet is received or when
    /// LFS is not in multiplayer mode (empty host name in the `ISM`).
    pub fn multiplayer(&self) -> &MultiplayerState {
        &self.multiplayer
    }

    /// Server's allowed-cars set, if a `Small`/`Alc` has been received.
    ///
    /// Populated by requesting `TinyType::Alc`, or when the host changes the
    /// allowed cars mid-session.
    pub fn allowed_cars(&self) -> Option<&PlcAllowedCarsSet> {
        self.allowed_cars.as_ref()
    }

    /// Server's allowed-mods list, if a `Mal` has been received. Empty means
    /// unrestricted (any mod) or not yet received.
    ///
    /// Populated by requesting `TinyType::Mal`, or when the host changes the
    /// allowed mods mid-session.
    pub fn allowed_mods(&self) -> &[Vehicle] {
        &self.allowed_mods
    }

    /// Version information about the connected LFS instance, if a `Ver` has
    /// been received. LFS sends this on connect when `Isi::reqi` is non-zero,
    /// or in reply to `TinyType::Ver`.
    pub fn version(&self) -> Option<&VersionInfo> {
        self.version.as_ref()
    }
}

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
