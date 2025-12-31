//! Vehicle Truck object
use crate::{
    DecodeError,
    heading::Heading,
    object::{ObjectCoordinate, ObjectFlags},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Cone Colour
pub enum VehicleTruckColour {
    #[default]
    Black,
    Orange,
    White,
    Yellow,
    DarkBlue,
    Red,
    LightBlue,
}

impl From<u8> for VehicleTruckColour {
    fn from(value: u8) -> Self {
        match value & 0x07 {
            0 => Self::Black,
            1 => Self::Orange,
            2 => Self::White,
            3 => Self::Yellow,
            4 => Self::DarkBlue,
            5 => Self::Red,
            6 => Self::LightBlue,
            _ => Self::Black,
        }
    }
}
/// Vehicle Truck
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VehicleTruck {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: VehicleTruckColour,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl VehicleTruck {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = self.colour as u8 & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        ObjectFlags(flags)
    }

    pub(super) fn new(
        xyz: ObjectCoordinate,
        wire: ObjectFlags,
        heading: Heading,
    ) -> Result<Self, DecodeError> {
        let colour = VehicleTruckColour::from(wire.colour());
        let mapping = wire.mapping();
        let floating = wire.floating();
        Ok(Self {
            xyz,
            heading,
            colour,
            mapping,
            floating,
        })
    }
}
