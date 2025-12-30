//! Bin1 object
use crate::{heading::Heading, object::{ObjectCoordinate, ObjectFlags}, DecodeError};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
#[allow(missing_docs)]
/// Bin Colour
pub enum Bin1Colour {
    #[default]
    Red,
    Yellow,
    Blue,
    Green,
    White,
    Orange,
}

impl From<u8> for Bin1Colour {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Red,
            1 => Self::Yellow,
            2 => Self::Blue,
            3 => Self::Green,
            4 => Self::White,
            5 => Self::Orange,
            _ => Self::Green,
        }
    }
}

/// Bin1
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Bin1 {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: Bin1Colour,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl Bin1 {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = self.colour as u8 & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        ObjectFlags(flags)
    }

    pub(super) fn new(xyz: ObjectCoordinate, flags: ObjectFlags, heading: Heading) -> Result<Self, DecodeError> {
        let colour = Bin1Colour::from(flags.colour());
        let mapping = flags.mapping();
        let floating = flags.floating();
        Ok(Self {
            xyz,
            heading,
            colour,
            mapping,
            floating,
        })
    }
}
