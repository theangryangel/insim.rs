//! Events produced by [`World::apply_packet`](crate::world::World::apply_packet).
//!
//! [`WorldEvent`] is the single aggregate event type. Each variant wraps a
//! distinct payload struct (defined here) so that frameworks dispatching by
//! concrete type - notably `kitcar`'s `Event<T>` extractor - can re-export and
//! match on the individual structs.

use insim::{
    core::{game_version::GameVersion, track::Track, vehicle::Vehicle},
    identifiers::ConnectionId,
    insim::PlcAllowedCarsSet,
};

use crate::world::{
    connection::{ConnectionInfo, PlayerInfo},
    game::SessionKind,
    race::RaceEvent,
};

/// A new connection joined.
#[derive(Debug, Clone)]
pub struct Connected(pub ConnectionInfo);

/// A connection left.
#[derive(Debug, Clone)]
pub struct Disconnected {
    /// The connection that left.
    pub ucid: ConnectionId,
    /// Last known info (cloned before removal).
    pub info: Option<ConnectionInfo>,
}

/// Extra connection details arrived via `Nci`.
#[derive(Debug, Clone)]
pub struct ConnectionDetails(pub ConnectionInfo);

/// A connection selected a vehicle in the garage.
#[derive(Debug, Clone)]
pub struct VehicleSelected {
    /// The connection.
    pub ucid: ConnectionId,
    /// The selected vehicle.
    pub vehicle: Vehicle,
}

/// A connection changed their display name.
#[derive(Debug, Clone)]
pub struct Renamed {
    /// Connection ID.
    pub ucid: ConnectionId,
    /// Stable LFS.net username.
    pub uname: String,
    /// New display name.
    pub new_pname: String,
}

/// A player joined the track.
#[derive(Debug, Clone)]
pub struct PlayerJoined(pub PlayerInfo);

/// A player left the track.
#[derive(Debug, Clone)]
pub struct PlayerLeft(pub PlayerInfo);

/// A driver swap occurred.
#[derive(Debug, Clone)]
pub struct TakingOver {
    /// Player state before the swap.
    pub before: PlayerInfo,
    /// Player state after the swap.
    pub after: PlayerInfo,
}

/// A player tele-pitted (Shift+P).
#[derive(Debug, Clone)]
pub struct PlayerTeleportedToPits(pub PlayerInfo);

/// An `Rst` packet started a new race, qualifying, practice or untimed session.
#[derive(Debug, Clone)]
pub struct SessionStarted {
    /// The kind of session that started.
    pub kind: SessionKind,
}

/// LFS returned to the entry/lobby screen.
#[derive(Debug, Clone)]
pub struct SessionEnded;

/// Track changed (also fired for the first `Sta` when `from` is `None`).
#[derive(Debug, Clone)]
pub struct TrackChanged {
    /// Previously known track.
    pub from: Option<Track>,
    /// New track.
    pub to: Track,
}

/// Layout changed or cleared.
#[derive(Debug, Clone)]
pub struct LayoutChanged {
    /// Previously known layout.
    pub from: Option<String>,
    /// New layout, or `None` if cleared.
    pub to: Option<String>,
}

/// LFS joined or started a multiplayer session.
#[derive(Debug, Clone)]
pub struct MultiplayerJoined {
    /// Multiplayer host name.
    pub host_name: String,
    /// `true` if this instance is the host.
    pub is_host: bool,
}

/// LFS left multiplayer (received an `ISM` with an empty host name).
#[derive(Debug, Clone)]
pub struct MultiplayerLeft;

/// The server's allowed-cars set changed (from a `Small`/`Alc`).
#[derive(Debug, Clone)]
pub struct AllowedCarsChanged {
    /// The new allowed-cars set.
    pub cars: PlcAllowedCarsSet,
}

/// The server's allowed-mods list changed (from a `Mal`).
#[derive(Debug, Clone)]
pub struct AllowedModsChanged {
    /// The new allowed-mods list (empty means unrestricted).
    pub mods: Vec<Vehicle>,
}

/// Version information was received (from a `Ver`).
#[derive(Debug, Clone)]
pub struct VersionReceived {
    /// Product name (e.g. `"S3"`).
    pub product: String,
    /// LFS game version.
    pub version: GameVersion,
}

/// Aggregate event produced by [`World::apply_packet`](crate::world::World::apply_packet).
///
/// Each variant wraps the matching payload struct from this module; race events
/// travel as the nested [`RaceEvent`] enum.
#[derive(Debug, Clone)]
pub enum WorldEvent {
    /// A new connection joined.
    Connected(Connected),
    /// A connection left.
    Disconnected(Disconnected),
    /// Extra connection details arrived via `Nci`.
    ConnectionDetails(ConnectionDetails),
    /// A connection selected a vehicle in the garage.
    VehicleSelected(VehicleSelected),
    /// A connection changed their display name.
    Renamed(Renamed),
    /// A player joined the track.
    PlayerJoined(PlayerJoined),
    /// A player left the track.
    PlayerLeft(PlayerLeft),
    /// A driver swap occurred.
    TakingOver(TakingOver),
    /// A player tele-pitted (Shift+P).
    PlayerTeleportedToPits(PlayerTeleportedToPits),
    /// A new session started.
    SessionStarted(SessionStarted),
    /// LFS returned to the entry/lobby screen.
    SessionEnded(SessionEnded),
    /// Track changed.
    TrackChanged(TrackChanged),
    /// Layout changed or cleared.
    LayoutChanged(LayoutChanged),
    /// LFS joined or started a multiplayer session.
    MultiplayerJoined(MultiplayerJoined),
    /// LFS left multiplayer.
    MultiplayerLeft(MultiplayerLeft),
    /// The server's allowed-cars set changed.
    AllowedCarsChanged(AllowedCarsChanged),
    /// The server's allowed-mods list changed.
    AllowedModsChanged(AllowedModsChanged),
    /// Version information was received.
    VersionReceived(VersionReceived),
    /// A race event (entrant joined, lap completed, finished, etc.).
    Race(RaceEvent),
}
