//! Game-state types used by [`crate::world::World`].
//!
//! The [`GameEvent`] enum, [`SessionState`], [`SessionKind`], [`MultiplayerState`],
//! [`GameInfo`], and related types are defined here. The actual state is owned by
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
/// Carried by [`GameEvent::SessionStarted`] so consumers (notably the race
/// tracker) can interpret subsequent packets correctly - most importantly
/// `Fin`, which marks a finish in a race but fires after every lap in
/// qualifying.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    /// A race session (`Rst` with `qualmins == 0` and a lap/hour count).
    Race,
    /// A qualifying session (`Rst` with `qualmins > 0`).
    Qualifying,
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
        matches!(self, Self::Race)
    }

    /// Whether this session kind warrants lap and entrant tracking.
    ///
    /// Returns `true` for [`Race`](Self::Race), [`Qualifying`](Self::Qualifying),
    /// and [`Practice`](Self::Practice). Returns `false` for
    /// [`Untimed`](Self::Untimed) (custom game modes where race tracking is
    /// meaningless).
    pub fn is_tracking(self) -> bool {
        matches!(self, Self::Race | Self::Qualifying | Self::Practice)
    }
}

/// High-level description of the current LFS session.
#[derive(Debug, Default, Clone)]
pub enum SessionState {
    /// No `Sta` or `Rst` received yet; state unknown.
    #[default]
    Unknown,
    /// No session active - players are on the track-selection or end screen.
    Lobby,
    /// A race session is in progress.
    Racing {
        /// Race length configuration.
        laps: RaceLaps,
        /// Race flags from the `Rst` packet, or empty if only `Sta` has been received.
        flags: RaceFlags,
    },
    /// A qualifying session is in progress.
    Qualifying {
        /// Qualifying duration.
        duration: Duration,
        /// Race flags from the `Rst` packet, or empty if only `Sta` has been received.
        flags: RaceFlags,
    },
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
    pub(crate) session: SessionState,
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

    /// Current session state.
    pub fn session(&self) -> &SessionState {
        &self.session
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

/// State-change events produced by game packet processing in [`World::apply_packet`](crate::world::World::apply_packet).
///
/// Standalone users pattern-match this directly. `kitcar` users receive
/// individual typed events via `Event<T>` extractors.
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// An `Rst` packet started a new race, qualifying, practice or untimed
    /// session.
    ///
    /// This is the authoritative session-start signal: LFS sends an
    /// unsolicited `Rst` precisely when a session starts or restarts, carrying
    /// enough information to tell races, qualifying and practice/untimed apart
    /// (see [`SessionKind`]). Consumers that accumulate per-session state
    /// should clear it here.
    SessionStarted {
        /// The kind of session that started.
        kind: SessionKind,
    },
    /// LFS returned to the entry/lobby screen: an `Sta` reported no race or
    /// qualifying in progress where the previous state had one.
    ///
    /// This is the session-end signal - there is no `Rst`-based equivalent, as
    /// `Rst` only ever starts a session. Note that practice/untimed sessions
    /// are reported by LFS as "no race in progress", so this does not fire for
    /// them.
    SessionEnded,
    /// Track changed (also fired for the first `Sta` when `from` is `None`).
    TrackChanged {
        /// Previously known track.
        from: Option<Track>,
        /// New track.
        to: Track,
    },
    /// Layout changed or cleared.
    LayoutChanged {
        /// Previously known layout.
        from: Option<String>,
        /// New layout, or `None` if cleared.
        to: Option<String>,
    },
    /// LFS joined or started a multiplayer session.
    ///
    /// Fired on the first `ISM` with a non-empty host name, and again if the
    /// host name changes (e.g. reconnecting to a different server).
    MultiplayerJoined {
        /// Multiplayer host name.
        host_name: String,
        /// `true` if this instance is the host.
        is_host: bool,
    },
    /// LFS left multiplayer (received an `ISM` with an empty host name).
    MultiplayerLeft,
    /// The server's allowed-cars set changed (from a `Small`/`Alc`).
    ///
    /// Also fired the first time the set is received.
    AllowedCarsChanged {
        /// The new allowed-cars set.
        cars: PlcAllowedCarsSet,
    },
    /// The server's allowed-mods list changed (from a `Mal`).
    ///
    /// Also fired the first time the list is received.
    AllowedModsChanged {
        /// The new allowed-mods list (empty means unrestricted).
        mods: Vec<Vehicle>,
    },
    /// Version information was received (from a `Ver`).
    VersionReceived {
        /// Product name (e.g. `"S3"`).
        product: String,
        /// LFS game version.
        version: GameVersion,
    },
}
