//! Connections and players as [`World`](crate::world::World) sees them.
//!
//! Per-connection admin commands live on [`ConnectionInfo`]. Reach one with
//! [`World::connection`](crate::world::World::connection) or
//! [`World::connection_by_player`](crate::world::World::connection_by_player).

use std::net::Ipv4Addr;

use insim::{
    core::vehicle::Vehicle,
    identifiers::ConnectionId,
    insim::{Mst, PenaltyInfo, Plc, PlcAllowedCarsSet},
};

/// A connection, as tracked by [`World`](crate::world::World). Admin commands
/// return packets for you to send.
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// UCID.
    pub ucid: ConnectionId,
    /// LFS.net username.
    pub uname: String,
    /// Display name.
    pub pname: String,
    /// Has admin rights.
    pub admin: bool,
    /// LFS.net user ID. Needs an admin password on the host (via `Nci`).
    pub userid: Option<u32>,
    /// IP address. Needs an admin password on the host (via `Nci`).
    pub ipaddress: Option<Ipv4Addr>,
    /// Last vehicle picked in the garage (via `Slc`).
    pub selected_vehicle: Option<Vehicle>,
}

// XXX: `MultiIndexMap` generates undocumentable items, so `missing_docs` is
// off for this module only.
#[allow(missing_docs)]
mod player_info {
    use insim::{
        core::vehicle::Vehicle,
        identifiers::{ConnectionId, PlayerId},
        insim::{PlayerFlags, PlayerType},
    };
    use multi_index_map::MultiIndexMap;

    /// A player (a car on track), as tracked by [`crate::world::World`].
    #[derive(MultiIndexMap, Debug, Clone)]
    pub struct PlayerInfo {
        /// PLID.
        #[multi_index(hashed_unique)]
        pub plid: PlayerId,
        /// UCID of the controlling connection.
        #[multi_index(hashed_non_unique)]
        pub ucid: ConnectionId,
        /// Vehicle.
        pub vehicle: Vehicle,
        /// AI, female, remote and so on.
        pub ptype: PlayerType,
        /// Driver aids and setup flags.
        pub flags: PlayerFlags,
        /// In the pit lane.
        pub in_pitlane: bool,
        /// Display name at join.
        pub pname: String,
    }
}

pub(crate) use player_info::MultiIndexPlayerInfoMap;
pub use player_info::PlayerInfo;

impl ConnectionInfo {
    /// `/kick`.
    pub fn kick(&self) -> Mst {
        Mst::new(format!("/kick {}", self.uname))
    }

    /// `/ban`. Zero days means 12 hours.
    pub fn ban(&self, ban_days: u32) -> Mst {
        Mst::new(format!("/ban {} {ban_days}", self.uname))
    }

    /// `/spec`.
    pub fn spec(&self) -> Mst {
        Mst::new(format!("/spec {}", self.uname))
    }

    /// `/pitlane`.
    pub fn pitlane(&self) -> Mst {
        Mst::new(format!("/pitlane {}", self.uname))
    }

    /// `/p_clear`.
    pub fn clear_penalty(&self) -> Mst {
        Mst::new(format!("/p_clear {}", self.uname))
    }

    /// Issue a penalty. `None` for variants LFS has no command for.
    pub fn give_penalty(&self, penalty: PenaltyInfo) -> Option<Mst> {
        let cmd = match penalty {
            PenaltyInfo::Dt => format!("/p_dt {}", self.uname),
            PenaltyInfo::Sg => format!("/p_sg {}", self.uname),
            PenaltyInfo::Seconds30 => format!("/p_30 {}", self.uname),
            PenaltyInfo::Seconds45 => format!("/p_45 {}", self.uname),
            _ => return None,
        };
        Some(Mst::new(cmd))
    }

    /// Show a race control message. Send both packets, in order.
    pub fn send_rcm(&self, message: &str) -> [Mst; 2] {
        [
            Mst::new(format!("/rcm {message}")),
            Mst::new(format!("/rcm_ply {}", self.uname)),
        ]
    }

    /// Clear the race control message.
    pub fn clear_rcm(&self) -> Mst {
        Mst::new(format!("/rcc_ply {}", self.uname))
    }

    /// Restrict this connection to `vehicles`. Standard cars only: LFS allows
    /// or bans mods server-wide, so mods here are ignored.
    pub fn restrict_vehicles(&self, vehicles: impl IntoIterator<Item = Vehicle>) -> Plc {
        let mut cars = PlcAllowedCarsSet::default();
        for vehicle in vehicles {
            let _ = cars.insert(vehicle);
        }
        Plc {
            ucid: self.ucid,
            cars,
            ..Default::default()
        }
    }
}
