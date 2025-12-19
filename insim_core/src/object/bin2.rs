//! Bin2 object
use super::{ObjectVariant, ObjectWire};
use crate::{DecodeError, direction::Direction};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
#[allow(missing_docs)]
/// Post Colour
pub enum Bin2Colour {
    #[default]
    Green,
    Blue,
    Yellow,
    Black,
    White,
    Orange,
}

impl From<u8> for Bin2Colour {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Green,
            1 => Self::Blue,
            2 => Self::Yellow,
            3 => Self::Black,
            4 => Self::White,
            5 => Self::Orange,
            _ => Self::Green,
        }
    }
}

/// Bin2
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Bin2 {
    /// Heading / Direction
    pub heading: Direction,
    /// Colour (3 bits, 0-7)
    pub colour: Bin2Colour,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Bin2 {
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
        let colour = Bin2Colour::from(wire.colour());
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
    fn test_bin2_round_trip() {
        let original = Bin2::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = Bin2::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
