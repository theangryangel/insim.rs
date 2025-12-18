//! Tyre stack2 big object
use super::{ObjectVariant, ObjectWire};
use crate::{DecodeError, direction::Direction};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
/// Tyre Stack Colour
pub enum TyreStackColour {
    /// Black
    #[default]
    Black,
    /// White
    White,
    /// Red
    Red,
    /// Blue
    Blue,
    /// Green
    Green,
    /// Yellow
    Yellow,
}

impl From<u8> for TyreStackColour {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Black,
            1 => Self::White,
            2 => Self::Red,
            3 => Self::Blue,
            4 => Self::Green,
            5 => Self::Yellow,
            _ => Self::Black,
        }
    }
}

impl From<TyreStackColour> for u8 {
    fn from(colour: TyreStackColour) -> Self {
        match colour {
            TyreStackColour::Black => 0,
            TyreStackColour::White => 1,
            TyreStackColour::Red => 2,
            TyreStackColour::Blue => 3,
            TyreStackColour::Green => 4,
            TyreStackColour::Yellow => 5,
        }
    }
}

/// Tyre stack2 big
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TyreStack2Big {
    /// Colour
    pub colour: TyreStackColour,
    /// Heading / Direction
    pub heading: Direction,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for TyreStack2Big {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = u8::from(self.colour) & 0x07;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError> {
        let colour = TyreStackColour::from(wire.colour());
        let floating = wire.floating();
        Ok(Self {
            colour,
            heading: Direction::from_objectinfo_heading(wire.heading),
            floating,
        })
    }
}
