//! Control objects
use super::{ObjectCodec, ObjectPosition};

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

/// Tyre stack
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TyreStack {
    /// XYZ position
    pub xyz: ObjectPosition,
    /// Colour
    pub colour: TyreStackColour,
    /// Heading / Direction
    pub heading: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectCodec for TyreStack {
    fn encode(&self) -> Result<(&ObjectPosition, u8, u8), crate::EncodeError> {
        let mut flags = 0;
        flags |= self.colour as u8 & 0x07;
        if self.floating {
            flags |= 0x80;
        }
        Ok((&self.xyz, flags, self.heading))
    }

    fn decode(xyz: ObjectPosition, flags: u8, heading: u8) -> Result<Self, crate::DecodeError> {
        let colour = TyreStackColour::from(flags & 0x07);
        let floating = flags & 0x80 != 0;
        Ok(Self {
            xyz,
            colour,
            heading,
            floating,
        })
    }
}
