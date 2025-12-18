//! Chalk right object
use super::{ObjectVariant, ObjectWire};
use crate::{DecodeError, direction::Direction};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Chalk Colour
pub enum ChalkColour {
    #[default]
    White,
    Red,
    Blue,
    Yellow,
}

impl From<u8> for ChalkColour {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::White,
            1 => Self::Red,
            2 => Self::Blue,
            3 => Self::Yellow,
            _ => Self::White,
        }
    }
}

impl From<ChalkColour> for u8 {
    fn from(colour: ChalkColour) -> Self {
        match colour {
            ChalkColour::White => 0,
            ChalkColour::Red => 1,
            ChalkColour::Blue => 2,
            ChalkColour::Yellow => 3,
        }
    }
}

/// Chalk right
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ChalkRight {
    /// Colour
    pub colour: ChalkColour,
    /// Heading / Direction
    pub heading: Direction,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for ChalkRight {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = u8::from(self.colour) & 0x07;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            index: 11,
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError> {
        let colour = ChalkColour::from(wire.colour());
        let floating = wire.floating();
        Ok(Self {
            colour,
            heading: Direction::from_objectinfo_heading(wire.heading),
            floating,
        })
    }
}
