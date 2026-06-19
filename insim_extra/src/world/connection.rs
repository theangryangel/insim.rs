//! Connection and player types used by [`crate::world::World`].
//!
//! Per-connection admin commands are implemented on [`ConnectionInfo`]; reach
//! them via [`World::get`](crate::world::World::get) or
//! [`World::connection_by_player`](crate::world::World::connection_by_player).

use std::net::Ipv4Addr;

use insim::{core::vehicle::Vehicle, identifiers::ConnectionId, insim::PenaltyInfo};

use crate::util::host_command;

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

impl ConnectionInfo {
    /// Returns a `/kick` packet for this connection.
    pub fn kick(&self) -> insim::Packet {
        host_command(format!("/kick {}", self.uname))
    }

    /// Returns a `/ban` packet. `ban_days = 0` means 12 hours (LFS convention).
    pub fn ban(&self, ban_days: u32) -> insim::Packet {
        host_command(format!("/ban {} {ban_days}", self.uname))
    }

    /// Returns a `/spec` packet for this connection.
    pub fn spec(&self) -> insim::Packet {
        host_command(format!("/spec {}", self.uname))
    }

    /// Returns a `/pitlane` packet for this connection.
    pub fn pitlane(&self) -> insim::Packet {
        host_command(format!("/pitlane {}", self.uname))
    }

    /// Returns a `/p_clear` packet for this connection.
    pub fn clear_penalty(&self) -> insim::Packet {
        host_command(format!("/p_clear {}", self.uname))
    }

    /// Returns a penalty packet for this connection, or `None` for unissueable
    /// penalty variants.
    pub fn give_penalty(&self, penalty: PenaltyInfo) -> Option<insim::Packet> {
        let cmd = match penalty {
            PenaltyInfo::Dt => format!("/p_dt {}", self.uname),
            PenaltyInfo::Sg => format!("/p_sg {}", self.uname),
            PenaltyInfo::Seconds30 => format!("/p_30 {}", self.uname),
            PenaltyInfo::Seconds45 => format!("/p_45 {}", self.uname),
            _ => return None,
        };
        Some(host_command(cmd))
    }

    /// Returns the packets needed to set and display a Race Control Message for
    /// this connection. Send both packets.
    pub fn send_rcm(&self, message: &str) -> Vec<insim::Packet> {
        vec![
            host_command(format!("/rcm {message}")),
            host_command(format!("/rcm_ply {}", self.uname)),
        ]
    }

    /// Returns a packet to clear the Race Control Message for this connection.
    pub fn clear_rcm(&self) -> insim::Packet {
        host_command(format!("/rcc_ply {}", self.uname))
    }
}
