//! Synthetic event structs for presence changes, re-exported from `insim_extra`.

pub use insim_extra::presence::{ConnectionInfo, PlayerInfo};

/// Synthetic event emitted when a connection joins.
#[derive(Debug, Clone)]
pub struct Connected(pub ConnectionInfo);

/// Synthetic event emitted when a connection leaves.
#[derive(Debug, Clone)]
pub struct Disconnected {
    /// The connection that left.
    pub ucid: insim::identifiers::ConnectionId,
    /// Last known info for the connection.
    pub info: Option<ConnectionInfo>,
}

/// Synthetic event emitted when a connection changes their display name.
#[derive(Debug, Clone)]
pub struct Renamed {
    /// Connection ID.
    pub ucid: insim::identifiers::ConnectionId,
    /// LFS.net username (stable).
    pub uname: String,
    /// New display name.
    pub new_pname: String,
}

/// Synthetic event emitted when extra connection details arrive via `Nci`.
#[derive(Debug, Clone)]
pub struct ConnectionDetails(pub ConnectionInfo);

/// Synthetic event emitted when a connection selects a vehicle in the garage.
#[derive(Debug, Clone)]
pub struct VehicleSelected {
    /// Connection that selected the vehicle.
    pub ucid: insim::identifiers::ConnectionId,
    /// The selected vehicle.
    pub vehicle: insim::core::vehicle::Vehicle,
}

/// Synthetic event emitted when a player joins the track.
#[derive(Debug, Clone)]
pub struct PlayerJoined(pub PlayerInfo);

/// Synthetic event emitted when a player leaves the track.
#[derive(Debug, Clone)]
pub struct PlayerLeft(pub PlayerInfo);

/// Synthetic event emitted when a player's controlling connection changes.
#[derive(Debug, Clone)]
pub struct TakingOver {
    /// The player before the swap.
    pub before: PlayerInfo,
    /// The player after the swap.
    pub after: PlayerInfo,
}

/// Synthetic event emitted when a player tele-pits (Shift+P).
#[derive(Debug, Clone)]
pub struct PlayerTeleportedToPits(pub PlayerInfo);
