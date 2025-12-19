//! Vehicle SUV object
use super::{ObjectVariant, ObjectWire};
use crate::{DecodeError, direction::Direction};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Cone Colour
pub enum VehicleSUVColour {
    /// White
    #[default]
    White = 0,
    Red,
    LightBlue,
    Green,
    DarkBlue,
    Black,
    Orange,
    Yellow,
}

impl From<u8> for VehicleSUVColour {
    fn from(value: u8) -> Self {
        match value & 0x07 {
            0 => Self::White,
            1 => Self::Red,
            2 => Self::LightBlue,
            3 => Self::Green,
            4 => Self::DarkBlue,
            5 => Self::Black,
            6 => Self::Orange,
            7 => Self::Yellow,
            _ => Self::White,
        }
    }
}

/// Vehicle SUV
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VehicleSUV {
    /// Heading / Direction
    pub heading: Direction,
    /// Colour (3 bits, 0-7)
    pub colour: VehicleSUVColour,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for VehicleSUV {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = self.colour as u8 & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError> {
        let colour = VehicleSUVColour::from(wire.colour());
        let mapping = wire.mapping();
        let floating = wire.floating();
        Ok(Self {
            heading: Direction::from_objectinfo_heading(wire.heading),
            colour,
            mapping,
            floating,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_s_u_v_round_trip() {
        let original = VehicleSUV::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = VehicleSUV::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
