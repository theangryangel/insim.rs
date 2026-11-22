//! What the game is doing: the kind of session, the track and conditions,
//! multiplayer, and what the server allows.
//!
//! [`World`](crate::world::World) owns this state. [`GameInfo`] is the snapshot
//! it hands out, and [`SessionKind`] decides how race packets are read.

use std::time::Duration;

use insim::{
    core::{game_version::GameVersion, track::Track, vehicle::Vehicle, wind::Wind},
    insim::{PlcAllowedCarsSet, RaceFlags, RaceLaps, StaFlags},
};

/// The kind of session an `Rst` started.
///
/// It decides how later packets read. Above all `Fin`: a finish in a race, but
/// a per-lap signal in every other kind.
#[derive(Debug, Clone, Copy)]
pub enum SessionKind {
    /// A race (`Rst` with `qualmins == 0` and a lap or hour count).
    Race {
        /// Race length.
        laps: RaceLaps,
        /// Race flags from the `Rst`.
        flags: RaceFlags,
    },
    /// A qualifying session (`Rst` with `qualmins > 0`).
    Qualifying {
        /// Qualifying length.
        duration: Duration,
        /// Race flags from the `Rst`.
        flags: RaceFlags,
    },
    /// A practice session (`Rst` with `qualmins == 0` and
    /// [`RaceLaps::Practice`], i.e. 0 laps).
    ///
    /// LFS reports no race in progress, so only the `Rst` tells this apart.
    Practice,
    /// An untimed open or cruise session (`Rst` with `qualmins == 0` and
    /// [`RaceLaps::Untimed`]).
    ///
    /// As with [`Practice`](Self::Practice), LFS reports no race in progress.
    Untimed,
}

impl SessionKind {
    /// Whether this is a race, where `Fin` is a finish and leaving early is a DNF.
    pub fn is_race(self) -> bool {
        matches!(self, Self::Race { .. })
    }

    /// Whether laps and entrants are tracked: yes for [`Race`](Self::Race) and
    /// [`Qualifying`](Self::Qualifying), no for [`Practice`](Self::Practice) and
    /// [`Untimed`](Self::Untimed).
    pub fn is_tracking(self) -> bool {
        matches!(self, Self::Race { .. } | Self::Qualifying { .. })
    }
}

/// Whether LFS is offline or in multiplayer.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum MultiplayerState {
    /// Single-player or offline.
    #[default]
    Local,
    /// In a multiplayer session.
    Multiplayer {
        /// Server host name.
        host_name: String,
        /// Whether this instance is the host rather than a guest.
        is_host: bool,
    },
}

/// The connected LFS instance's version, from `Ver`.
#[derive(Debug, Default, Clone)]
pub struct VersionInfo {
    /// Product name (e.g. `"S3"`).
    pub product: String,
    /// LFS game version.
    pub version: GameVersion,
}

/// Snapshot of the game state, from [`World::game_info()`](crate::world::World::game_info).
#[derive(Debug, Default, Clone)]
pub struct GameInfo {
    /// `None` outside a session, or before the first `STA`.
    pub(crate) session_kind: Option<SessionKind>,
    pub(crate) track: Option<Track>,
    pub(crate) layout: Option<String>,
    pub(crate) weather: Option<u8>,
    pub(crate) wind: Option<Wind>,
    pub(crate) flags: StaFlags,
    pub(crate) multiplayer: MultiplayerState,
    /// From `Small`/`Alc`. `None` until seen.
    pub(crate) allowed_cars: Option<PlcAllowedCarsSet>,
    /// From `Mal`. Empty means unrestricted, or not yet received.
    pub(crate) allowed_mods: Vec<Vehicle>,
    /// From `Ver`. `None` until received.
    pub(crate) version: Option<VersionInfo>,
}

impl GameInfo {
    /// Current session kind. `None` outside a session, or before the first
    /// `STA` arrives.
    pub fn session(&self) -> Option<SessionKind> {
        self.session_kind
    }

    /// Selected track, if known.
    pub fn current_track(&self) -> Option<&Track> {
        self.track.as_ref()
    }

    /// Loaded layout, if known.
    pub fn current_layout(&self) -> Option<&String> {
        self.layout.as_ref()
    }

    /// Weather identifier, usually 0..=2.
    pub fn weather(&self) -> Option<u8> {
        self.weather
    }

    /// Wind conditions.
    pub fn wind(&self) -> Option<&Wind> {
        self.wind.as_ref()
    }

    /// Game flags.
    pub fn flags(&self) -> &StaFlags {
        &self.flags
    }

    /// Multiplayer state. [`MultiplayerState::Local`] until an `ISM` arrives,
    /// or when its host name is empty.
    pub fn multiplayer(&self) -> &MultiplayerState {
        &self.multiplayer
    }

    /// Server's allowed cars, once a `Small`/`Alc` arrives: in reply to
    /// `TinyType::Alc`, or when the host changes them.
    pub fn allowed_cars(&self) -> Option<&PlcAllowedCarsSet> {
        self.allowed_cars.as_ref()
    }

    /// Server's allowed mods, from `Mal`: in reply to `TinyType::Mal`, or when
    /// the host changes them. Empty means any mod, or not yet received.
    pub fn allowed_mods(&self) -> &[Vehicle] {
        &self.allowed_mods
    }

    /// LFS version, once a `Ver` arrives. LFS sends one on connect when
    /// `Isi::reqi` is non-zero, or in reply to `TinyType::Ver`.
    pub fn version(&self) -> Option<&VersionInfo> {
        self.version.as_ref()
    }
}
