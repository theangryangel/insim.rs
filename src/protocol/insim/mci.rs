use crate::packet_flags;
use crate::{conversion, protocol::position::FixedPoint};
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

packet_flags! {
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct CompCarInfo: u8 {
        BLUE_FLAG => (1 << 0),
        YELLOW_FLAG => (1 << 1),
        LAGGING => (1 << 5),
        FIRST => (1 << 6),
        LAST => (1 << 7),
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Used within the [Mci] packet info field.
pub struct CompCar {
    /// Index of the last "node" that the player passed through.
    pub node: u16,

    /// The player's current lap.
    pub lap: u16,

    /// The current player's ID.
    pub plid: u8,

    /// Race position
    pub position: u8,

    #[deku(pad_bytes_after = "1")]
    pub info: CompCarInfo,

    /// Positional information for the player, in game units.
    pub xyz: FixedPoint,

    /// Speed in game world units (32768 = 100 m/s)
    /// You may use the speed_as_mph, speed_as_kmph and speed_as_mps functions to convert to real-world units.
    pub speed: u16,

    /// Direction of car's motion : 0 = world y direction, 32768 = 180 deg
    pub direction: u16,

    /// Direction of forward axis : 0 = world y direction, 32768 = 180 deg
    pub heading: u16,

    /// Signed, rate of change of heading : (16384 = 360 deg/s)
    pub angvel: i16,
}

impl CompCar {
    /// Converts game world speed to miles per hour.
    pub fn speed_as_mph(&self) -> f32 {
        conversion::speed::to_mph(self.speed)
    }

    /// Converts gameword speed to kilometers per hour.
    pub fn speed_as_kmph(&self) -> f32 {
        conversion::speed::to_kmph(self.speed)
    }

    /// Converts game world speed to meters per second.
    pub fn speed_as_mps(&self) -> f32 {
        conversion::speed::to_mps(self.speed)
    }

    /// Converts angvel into degrees per second.
    pub fn angvel_as_dps(&self) -> f32 {
        // 16384/360 = 45.511
        (self.angvel as f32) / 45.511
    }

    /// Convert direction to degrees.
    pub fn direction_as_deg(&self) -> f32 {
        conversion::directional::to_degrees(self.direction)
    }

    /// Convert heading to degrees.
    pub fn heading_as_deg(&self) -> f32 {
        conversion::directional::to_degrees(self.heading)
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Multi Car Info - positional information for players/vehicles.
/// The MCI packet does not contain the positional information for all players. Only some. The
/// maximum number of players depends on the version of Insim.
pub struct Mci {
    pub reqi: u8,

    pub numc: u8,

    #[deku(count = "numc")]
    pub info: Vec<CompCar>,
}
