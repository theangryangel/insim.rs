//! Banner objects
use crate::{
    heading::Heading,
    object::{ObjectCoordinate, ObjectFlags},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Chalk Colour
pub enum BannerColour {
    #[default]
    White = 0,
    Red,
    Yellow,
    Green,
    Blue,
    Black,
}

impl From<u8> for BannerColour {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::White,
            1 => Self::Red,
            2 => Self::Yellow,
            3 => Self::Green,
            4 => Self::Blue,
            5 => Self::Black,
            _ => Self::White,
        }
    }
}

/// Banner
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Banner {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: BannerColour,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl Banner {
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
        flags: ObjectFlags,
        heading: Heading,
    ) -> Result<Self, crate::DecodeError> {
        let colour = BannerColour::from(flags.colour());
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
