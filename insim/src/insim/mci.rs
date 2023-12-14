use insim_core::{
    binrw::{self, binrw},
    identifiers::{PlayerId, RequestId},
    point::Point,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    pub struct CompCarInfo: u8 {
        const BLUE_FLAG = (1 << 0);
        const YELLOW_FLAG = (1 << 1);
        const LAGGING = (1 << 5);
        const FIRST = (1 << 6);
        const LAST = (1 << 7);
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Multi Car Info - positional information for players/vehicles.
/// The MCI packet does not contain the positional information for all players. Only some. The
/// maximum number of players depends on the version of Insim.
pub struct Mci {
    pub reqi: RequestId,

    #[bw(calc = info.len() as u8)]
    numc: u8,

    #[br(count = numc)]
    pub info: Vec<CompCar>,
}
