//! Game-state types used by [`crate::world::World`].
//!
//! The [`SessionKind`], [`MultiplayerState`], [`GameInfo`], and related types
//! are defined here. The actual state is owned by
//! [`World`](crate::world::World), which exposes query methods directly.

mod commands;
use std::time::Duration;

pub use commands::{GridMode, Month, TimeDemoPreset, TimeSet};
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
