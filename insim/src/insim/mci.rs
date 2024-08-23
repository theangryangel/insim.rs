use bitflags::bitflags;
use insim_core::{
    binrw::{self, binrw},
    point::Point,
};

use crate::identifiers::{PlayerId, RequestId};

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    /// Additional Car Info.
    pub struct CompCarInfo: u8 {
        /// This car is in the way of a driver who is a lap ahead
        const BLUE = (1 << 0);

        /// This car is slow or stopped and in a dangerous place
        const YELLOW = (1 << 1);

        /// This car is lagging (missing or delayed position packets)
        const LAG = (1 << 5);

        /// This is the first compcar in this set of MCI packets
        const FIRST = (1 << 6);

        /// This is the last compcar in this set of MCI packets
        const LAST = (1 << 7);
    }
}

generate_bitflag_helpers! {
    CompCarInfo,

    pub has_blue_flag => BLUE,
    pub has_yellow_flag => YELLOW,
    pub is_lagging => LAG,
    pub is_first => FIRST,
    pub is_last => LAST
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Used within the [Mci] packet info field.
pub struct CompCar {
    /// Index of the last "node" that the player passed through.
    pub node: u16,

    /// The player's current lap.
    pub lap: u16,

    /// The current player's ID.
    pub plid: PlayerId,

    /// Race position
    pub position: u8,

    #[brw(pad_after = 1)]
    /// Additional information that describes this particular Compcar.
    pub info: CompCarInfo,

    /// Positional information for the player, in game units.
    pub xyz: Point<i32>,

    /// Speed in game world units (32768 = 100 m/s)
    /// You may use the speed_uom function to convert this to real world units if the uom feature
    /// is enabled.
    pub speed: u16,

    /// Direction of car's motion : 0 = world y direction, 32768 = 180 deg
    /// You may use the direction_uom function to convert this to real world units if the uom feature is enabled.
    pub direction: u16,

    /// Direction of forward axis : 0 = world y direction, 32768 = 180 deg
    /// You may use the heading_uom function to convert this to real world units if the uom feature is enabled.
    pub heading: u16,

    /// Signed, rate of change of heading : (16384 = 360 deg/s)
    /// You may use the angvel_uom function to convert this to real world units if the uom feature is enabled.
    pub angvel: i16,
}

impl CompCar {
    /// This is the first compcar in this set of MCI packets
    pub fn is_first(&self) -> bool {
        self.info.is_first()
    }

    /// This is the last compcar in this set of MCI packets
    pub fn is_last(&self) -> bool {
        self.info.is_last()
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Multi Car Info - positional information for players/vehicles.
/// The MCI packet does not contain the positional information for all players. Only some. The
/// maximum number of players depends on the version of Insim.
pub struct Mci {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    #[bw(calc = info.len() as u8)]
    numc: u8,

    /// Node and lap for a subset of players. Not all players may be included in a single packet.
    #[br(count = numc)]
    pub info: Vec<CompCar>,
}

impl Mci {
    /// Is this the first MCI packet in this set of MCI packets?
    pub fn is_first(&self) -> bool {
        self.info.iter().any(|i| i.is_first())
    }

    /// Is this the last MCI packet in this set of MCI packets?
    pub fn is_last(&self) -> bool {
        self.info.iter().any(|i| i.is_last())
    }
}
