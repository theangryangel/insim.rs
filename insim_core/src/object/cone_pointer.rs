//! Cone Pointer objects
use super::{ObjectVariant, ObjectWire};
use crate::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Cone Pointer Colour
pub enum ConePointerColour {
    /// Blue
    #[default]
    Blue = 2,
    /// Green
    Green = 3,
    /// Yellow
    Yellow = 6,
}

impl From<u8> for ConePointerColour {
    fn from(value: u8) -> Self {
        match value & 0x07 {
            2 => Self::Blue,
            3 => Self::Green,
            6 => Self::Yellow,
            _ => Self::Blue,
        }
    }
}

/// Cone Pointer
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConePointer {
    /// Colour
    pub colour: ConePointerColour,
    /// Heading / Direction
    pub heading: Direction,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for ConePointer {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= self.colour as u8 & 0x07;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let colour = ConePointerColour::from(wire.colour());
        let floating = wire.floating();
        Ok(Self {
            colour,
            heading: Direction::from_objectinfo_heading(wire.heading),
            floating,
        })
    }
}
