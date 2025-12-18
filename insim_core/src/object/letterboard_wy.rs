//! Letterboard WY (White/Yellow) objects
use super::{ObjectVariant, ObjectWire};
use crate::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Letterboard WY Colour
pub enum LetterboardWYColour {
    /// White
    #[default]
    White = 0,
    /// Yellow
    Yellow = 1,
}

impl From<u8> for LetterboardWYColour {
    fn from(value: u8) -> Self {
        match value & 0x01 {
            0 => Self::White,
            _ => Self::Yellow,
        }
    }
}

/// Letterboard WY (White/Yellow)
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LetterboardWY {
    /// Colour
    pub colour: LetterboardWYColour,
    /// Heading / Direction
    pub heading: Direction,
    /// Mapping (6 bits, 0-63)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for LetterboardWY {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let index = 92;
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
        let colour = LetterboardWYColour::from(wire.flags);
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
