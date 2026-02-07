//! Tyre single object
use crate::{
    DecodeError,
    heading::Heading,
    object::{ObjectCoordinate, ObjectFlags},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
/// Tyre Stack Colour
pub enum TyreColour {
    /// Black
    #[default]
    Black = 0,
    /// White
    White = 1,
    /// Red
    Red = 2,
    /// Blue
    Blue = 3,
    /// Green
    Green = 4,
    /// Yellow
    Yellow = 5,
}

impl From<u8> for TyreColour {
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

/// Tyre single
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tyres {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Colour
    pub colour: TyreColour,
    /// Heading / Direction
    pub heading: Heading,
    /// Floating
    pub floating: bool,
}

impl Tyres {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = self.colour as u8 & 0x07;
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
        let colour = TyreColour::from(wire.colour());
        let floating = wire.floating();
        Ok(Self {
            xyz,
            colour,
            heading,
            floating,
        })
    }
}
