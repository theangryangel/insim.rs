//! Game-state event types re-exported from [`insim_extra::world`].
//!
//! Application workflows compose these events with ordinary handlers; Kitcar
//! intentionally provides no built-in round, track, or layout orchestration.

pub use insim_extra::world::{
    AllowedCarsChanged, AllowedModsChanged, GameInfo, LayoutChanged, LobbyEntered, LobbyNotReady,
    LobbyReady, MultiplayerJoined, MultiplayerLeft, SessionEnded, SessionKind, SessionStarted,
    TrackChanged, VersionInfo, VersionReceived,
};
