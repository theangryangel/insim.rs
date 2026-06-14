//! Connection and player types used by [`crate::world::World`].
//!
//! Admin commands are implemented on [`ConnectionInfo`] and re-exposed by
//! [`World`](crate::world::World) methods.

mod commands;

use std::net::Ipv4Addr;

use insim::{core::vehicle::Vehicle, identifiers::ConnectionId};

/// Per-connection record stored by [`World`](crate::world::World).
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Connection identifier.
    pub ucid: ConnectionId,
    /// LFS.net username.
    pub uname: String,
    /// Player nickname (display name).
    pub pname: String,
    /// Whether the connection has admin privileges.
    pub admin: bool,
    /// LFS.net user ID. Only populated when the host has an admin password
    /// set and an `Nci` packet has been received for this connection.
    pub userid: Option<u32>,
    /// Originating IP address. Only populated on hosts with an admin
    /// password (via `Nci`).
    pub ipaddress: Option<Ipv4Addr>,
    /// Most recently selected vehicle in the garage (from `Slc`).
    pub selected_vehicle: Option<Vehicle>,
}

// XXX: `MultiIndexMap` generates a map struct and associated methods that cannot
// carry doc comments, so `missing_docs` is suppressed for this module only.
#[allow(missing_docs)]
mod player_info {
    use insim::{
        core::vehicle::Vehicle,
        identifiers::{ConnectionId, PlayerId},
        insim::{PlayerFlags, PlayerType},
    };
    use multi_index_map::MultiIndexMap;

    /// Per-player record stored by [`crate::world::World`].
    #[derive(MultiIndexMap, Debug, Clone)]
    pub struct PlayerInfo {
        /// Player ID.
        #[multi_index(hashed_unique)]
        pub plid: PlayerId,
        /// Owning connection ID.
        #[multi_index(hashed_non_unique)]
        pub ucid: ConnectionId,
        /// Vehicle in use.
        pub vehicle: Vehicle,
        /// Player type flags (AI, female, remote, etc.).
        pub ptype: PlayerType,
        /// Player flags (pit-stop done, swap-out allowed, etc.).
        pub flags: PlayerFlags,
        /// Whether the player is currently in the pit lane.
        pub in_pitlane: bool,
        /// Player nickname at the moment of join.
        pub pname: String,
    }
}

pub(crate) use player_info::MultiIndexPlayerInfoMap;
pub use player_info::PlayerInfo;
