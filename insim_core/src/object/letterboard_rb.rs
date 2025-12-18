//! Letterboard RB (Red/Blue) objects
use super::{ObjectVariant, ObjectWire};
use crate::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Letterboard RB Colour
pub enum LetterboardRBColour {
    /// Red
    #[default]
    Red = 0,
    /// Blue
    Blue = 1,
}

impl From<u8> for LetterboardRBColour {
    fn from(value: u8) -> Self {
        match value & 0x01 {
            0 => Self::Red,
            _ => Self::Blue,
        }
    }
}

/// Letterboard RB (Red/Blue)
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LetterboardRB {
    /// Colour
    pub colour: LetterboardRBColour,
    /// Heading / Direction
    pub heading: Direction,
    /// Mapping (6 bits, 0-63)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for LetterboardRB {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let index = 93;
        let mut flags = self.colour as u8 & 0x01;
        flags |= (self.mapping & 0x3f) << 1;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            index,
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let colour = LetterboardRBColour::from(wire.flags);
        let mapping = (wire.flags >> 1) & 0x3f;
        let floating = wire.floating();
        Ok(Self {
            colour,
            heading: Direction::from_objectinfo_heading(wire.heading),
            mapping,
            floating,
        })
    }
}
