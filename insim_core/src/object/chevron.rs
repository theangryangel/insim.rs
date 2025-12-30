//! Cone1 objects
use crate::{heading::Heading, object::{ObjectCoordinate, ObjectFlags}};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Chevron Colour
pub enum ChevronColour {
    /// White
    #[default]
    White = 0,
    /// Black
    Black = 1,
}

impl From<u8> for ChevronColour {
    fn from(value: u8) -> Self {
        match value & 0x07 {
            0 => Self::White,
            1 => Self::Black,
            _ => Self::White,
        }
    }
}

/// Chevron
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Chevron {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Colour
    pub colour: ChevronColour,
    /// Heading / Direction
    pub heading: Heading,
    /// Floating
    pub floating: bool,
}

impl Chevron {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = 0;
        flags |= self.colour as u8 & 0x07;
        if self.floating {
            flags |= 0x80;
        }
        ObjectFlags(flags)
    }

    pub(super) fn from_wire(xyz: ObjectCoordinate, flags: ObjectFlags, heading: Heading) -> Result<Self, crate::DecodeError> {
        let colour = ChevronColour::from(flags.colour());
        let floating = flags.floating();
        Ok(Self {
            xyz,
            colour,
            heading,
            floating,
        })
    }
}
