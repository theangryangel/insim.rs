//! What changed, as reported by
//! [`World::apply`](crate::world::World::apply).
//!
//! [`WorldEvent`] wraps one payload struct per variant. The structs stand on
//! their own so code can ask for exactly the one it cares about, as kitcar's
//! [`Router::on`](crate::kitcar::services::router::Router::on) does.

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

/// The world has caught up with the server. Every startup reply is in, so it
/// knows the connections, players, track, layout and allowed cars that were
/// there before you connected.
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct WorldReady;

/// A connection the world hadn't seen. `existing` marks one that was already
/// there before we connected (a reply to a startup request), so greet only
/// those with `existing: false`.
#[derive(Debug, Clone)]
pub struct Connected {
    /// The connection.
    pub info: ConnectionInfo,
    /// Already connected before we were.
    pub existing: bool,
}

/// A connection left.
#[derive(Debug, Clone)]
pub struct Disconnected {
    /// The connection that left.
    pub ucid: ConnectionId,
    /// Last known details, if any.
    pub info: Option<ConnectionInfo>,
}

/// Extra connection details, from `Nci`.
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
    /// The connection.
    pub ucid: ConnectionId,
    /// LFS.net username, which never changes.
    pub uname: String,
    /// New display name.
    pub new_pname: String,
}

/// A player the world hadn't seen joined the track, or rejoined it. `existing`
/// marks one already on track before we connected.
#[derive(Debug, Clone)]
pub struct PlayerJoined {
    /// The player.
    pub info: PlayerInfo,
    /// Already on track before we connected.
    pub existing: bool,
}

/// A player left the track.
#[derive(Debug, Clone)]
pub struct PlayerLeft(pub PlayerInfo);

/// A driver swap.
#[derive(Debug, Clone)]
pub struct TakingOver {
    /// Before the swap.
    pub before: PlayerInfo,
    /// After the swap.
    pub after: PlayerInfo,
}

/// A player tele-pitted (Shift+P).
#[derive(Debug, Clone)]
pub struct PlayerTeleportedToPits(pub PlayerInfo);

/// An `Rst` started a race, qualifying, practice or untimed session.
#[derive(Debug, Clone)]
pub struct SessionStarted {
    /// The kind of session that started.
    pub kind: SessionKind,
}

/// The session ended: LFS went back to the lobby, or a `/restart` is about to
/// start a new one (a [`SessionStarted`] follows).
#[derive(Debug, Clone)]
pub struct SessionEnded;

/// LFS moved from an active session to the lobby.
#[derive(Debug, Clone)]
pub struct LobbyEntered;

/// The lobby is actionable: at least one non-server client is connected.
#[derive(Debug, Clone)]
pub struct LobbyReady;

/// The lobby stopped being actionable.
#[derive(Debug, Clone)]
pub struct LobbyNotReady;

/// The track changed. The first `Sta` fires this with `from: None`.
#[derive(Debug, Clone)]
pub struct TrackChanged {
    /// Previous track, if known.
    pub from: Option<Track>,
    /// New track.
    pub to: Track,
}

/// Layout changed or cleared.
#[derive(Debug, Clone)]
pub struct LayoutChanged {
    /// Previous layout, if known.
    pub from: Option<String>,
    /// New layout, or `None` if cleared.
    pub to: Option<String>,
}

/// LFS joined or started a multiplayer session.
#[derive(Debug, Clone)]
pub struct MultiplayerJoined {
    /// Multiplayer host name.
    pub host_name: String,
    /// Whether this instance is the host.
    pub is_host: bool,
}

/// LFS left multiplayer (an `ISM` with an empty host name).
#[derive(Debug, Clone)]
pub struct MultiplayerLeft;

/// The server's allowed cars changed (from `Small`/`Alc`).
#[derive(Debug, Clone)]
pub struct AllowedCarsChanged {
    /// The new set.
    pub cars: PlcAllowedCarsSet,
}

/// The server's allowed mods changed (from `Mal`).
#[derive(Debug, Clone)]
pub struct AllowedModsChanged {
    /// The new list. Empty means unrestricted.
    pub mods: Vec<Vehicle>,
}

/// LFS version info arrived (from `Ver`).
#[derive(Debug, Clone)]
pub struct VersionReceived {
    /// Product name (e.g. `"S3"`).
    pub product: String,
    /// LFS game version.
    pub version: GameVersion,
}

/// Everything [`World::apply`](crate::world::World::apply) can report.
///
/// Each variant wraps the payload struct of the same name. Race events nest as
/// [`RaceEvent`].
#[derive(Debug, Clone)]
pub enum WorldEvent {
    /// A connection joined.
    Connected(Connected),
    /// A connection left.
    Disconnected(Disconnected),
    /// Extra connection details arrived.
    ConnectionDetails(ConnectionDetails),
    /// A connection picked a vehicle.
    VehicleSelected(VehicleSelected),
    /// A connection changed their name.
    Renamed(Renamed),
    /// A player joined the track.
    PlayerJoined(PlayerJoined),
    /// A player left the track.
    PlayerLeft(PlayerLeft),
    /// A driver swap.
    TakingOver(TakingOver),
    /// A player tele-pitted.
    PlayerTeleportedToPits(PlayerTeleportedToPits),
    /// A new session started.
    SessionStarted(SessionStarted),
    /// A session ended.
    SessionEnded(SessionEnded),
    /// LFS moved from a session to the lobby.
    LobbyEntered(LobbyEntered),
    /// The lobby became actionable.
    LobbyReady(LobbyReady),
    /// The lobby stopped being actionable.
    LobbyNotReady(LobbyNotReady),
    /// Track changed.
    TrackChanged(TrackChanged),
    /// Layout changed or cleared.
    LayoutChanged(LayoutChanged),
    /// LFS joined or started multiplayer.
    MultiplayerJoined(MultiplayerJoined),
    /// LFS left multiplayer.
    MultiplayerLeft(MultiplayerLeft),
    /// The server's allowed cars changed.
    AllowedCarsChanged(AllowedCarsChanged),
    /// The server's allowed mods changed.
    AllowedModsChanged(AllowedModsChanged),
    /// LFS version info arrived.
    VersionReceived(VersionReceived),
    /// Something happened in the race: a lap, a pit stop, a finish.
    Race(RaceEvent),
    /// Every reply to [`World::STARTUP_REQUESTS`](crate::world::World::STARTUP_REQUESTS)
    /// is in. `apply` never emits this; whatever drives the world does
    /// (`World::attach`, or kitcar's runtime).
    Ready(WorldReady),
}
