//! Control objects
use super::{ObjectCodec, ObjectPosition};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Chalk Colour
pub enum ChalkColour {
    #[default]
    White = 0,
    Red = 1,
    Blue = 2,
    Yellow = 3,
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

/// Tyre stack
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Chalk {
    /// XYZ position
    pub xyz: ObjectPosition,
    /// Colour
    pub colour: ChalkColour,
    /// Heading / Direction
    pub heading: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectCodec for Chalk {
    fn encode(&self) -> Result<(&ObjectPosition, u8, u8), crate::EncodeError> {
        let mut flags = 0;
        flags |= self.colour as u8 & 0x07;
        if self.floating {
            flags |= 0x80;
        }
        Ok((&self.xyz, flags, self.heading))
    }

    fn decode(xyz: ObjectPosition, flags: u8, heading: u8) -> Result<Self, crate::DecodeError> {
        let colour = ChalkColour::from(flags & 0x07);
        let floating = flags & 0x80 != 0;
        Ok(Self {
            xyz,
            colour,
            heading,
            floating,
        })
    }
}
