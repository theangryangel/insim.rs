#[cfg(feature = "uom")]
use crate::units;

use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;
use crate::protocol::identifiers::{PlayerId, RequestId};
use crate::protocol::position::Point;

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct CompCarInfo: u8 {
        BLUE_FLAG => (1 << 0),
        YELLOW_FLAG => (1 << 1),
        LAGGING => (1 << 5),
        FIRST => (1 << 6),
        LAST => (1 << 7),
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
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

    #[insim(pad_bytes_after = "1")]
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

#[cfg(feature = "uom")]
impl CompCar {
    /// Converts speed into uom::si::f64::velocity
    pub fn speed_uom(&self) -> uom::si::f64::Velocity {
        uom::si::f64::Velocity::new::<units::velocity::game_per_second>(self.speed.into())
    }

    /// Converts angvel into degrees per second.
    pub fn angvel_uom(&self) -> uom::si::f64::AngularVelocity {
        uom::si::f64::AngularVelocity::new::<units::angular_velocity::game_heading_per_second>(
            self.speed.into(),
        )
    }

    /// Convert direction to degrees.
    pub fn direction_uom(&self) -> uom::si::f64::Angle {
        uom::si::f64::Angle::new::<units::angle::game_heading>(self.direction.into())
    }

    /// Convert direction to degrees.
    pub fn heading_uom(&self) -> uom::si::f64::Angle {
        uom::si::f64::Angle::new::<units::angle::game_heading>(self.heading.into())
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Multi Car Info - positional information for players/vehicles.
/// The MCI packet does not contain the positional information for all players. Only some. The
/// maximum number of players depends on the version of Insim.
pub struct Mci {
    pub reqi: RequestId,

    pub numc: u8,

    #[insim(count = "numc")]
    pub info: Vec<CompCar>,
}
