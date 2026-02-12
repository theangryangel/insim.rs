//! Vehicle Truck object
use crate::{
    DecodeError,
    heading::Heading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub(super) fn new(raw: Raw) -> Result<Self, DecodeError> {
        let xyz = raw.xyz;
        let heading = Heading::from_objectinfo_wire(raw.heading);
        let colour = VehicleTruckColour::from(raw.raw_colour());
        let mapping = raw.raw_mapping();
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            heading,
            colour,
            mapping,
            floating,
        })
    }
}
impl ObjectInfoInner for VehicleTruck {
    fn flags(&self) -> u8 {
        let mut flags = self.colour as u8 & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        flags
    }

    fn heading_mut(&mut self) -> Option<&mut Heading> {
        Some(&mut self.heading)
    }

    fn heading(&self) -> Option<Heading> {
        Some(self.heading)
    }

    fn floating(&self) -> Option<bool> {
        Some(self.floating)
    }

    fn heading_objectinfo_wire(&self) -> u8 {
        self.heading.to_objectinfo_wire()
    }
}
