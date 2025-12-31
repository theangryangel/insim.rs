//! Bin2 object
use crate::{
    DecodeError,
    heading::Heading,
    object::{ObjectCoordinate, ObjectFlags},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
#[allow(missing_docs)]
/// Bin Colour
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
    /// Position
    pub xyz: ObjectCoordinate,
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: Bin2Colour,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl Bin2 {
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
        let colour = Bin2Colour::from(wire.colour());
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
