//! Kerb objects
use crate::{
    heading::Heading,
    object::{ObjectCoordinate, ObjectFlags},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Kerb Mapping
pub enum KerbColour {
    /// White (light)
    #[default]
    White = 0,
    /// White (dark)
    WhiteDark = 1,
    /// Grey (light)
    Grey = 2,
    /// Grey (dark)
    GreyDark = 3,
    /// Red (light)
    Red = 4,
    /// Red (dark)
    RedDark = 5,
    /// Blue (light)
    Blue = 6,
    /// Blue (dark)
    BlueDark = 7,
    /// Cyan (light)
    Cyan = 8,
    /// Cyan (dark)
    CyanDark = 9,
    /// Green (light)
    Green = 10,
    /// Green (dark)
    GreenDark = 11,
    /// Orange (light)
    Orange = 12,
    /// Orange (dark)
    OrangeDark = 13,
    /// Yellow (light)
    Yellow = 14,
    /// Yellow (dark)
    YellowDark = 15,
}

impl From<u8> for KerbColour {
    fn from(value: u8) -> Self {
        match value & 0x0f {
            0 => Self::White,
            1 => Self::WhiteDark,
            2 => Self::Grey,
            3 => Self::GreyDark,
            4 => Self::Red,
            5 => Self::RedDark,
            6 => Self::Blue,
            7 => Self::BlueDark,
            8 => Self::Cyan,
            9 => Self::CyanDark,
            10 => Self::Green,
            11 => Self::GreenDark,
            12 => Self::Orange,
            13 => Self::OrangeDark,
            14 => Self::Yellow,
            15 => Self::YellowDark,
            _ => Self::White,
        }
    }
}

/// Kerb
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Kerb {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Mapping
    pub mapping: KerbColour,
    /// Floating
    pub floating: bool,
}

impl Kerb {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = self.colour & 0x07;
        flags |= (self.mapping as u8 & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        ObjectFlags(flags)
    }

    pub(super) fn new(
        xyz: ObjectCoordinate,
        wire: ObjectFlags,
        heading: Heading,
    ) -> Result<Self, crate::DecodeError> {
        let colour = wire.colour();
        let mapping = KerbColour::from(wire.mapping());
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
