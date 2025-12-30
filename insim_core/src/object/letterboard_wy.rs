//! Letterboard WY (White/Yellow) objects
use super::letterboard_rb::Character;
use crate::{heading::Heading, object::{ObjectCoordinate, ObjectFlags}};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Letterboard WY Colour
pub enum LetterboardWYColour {
    /// White
    #[default]
    White = 0,
    /// Yellow
    Yellow = 1,
}

impl From<u8> for LetterboardWYColour {
    fn from(value: u8) -> Self {
        match value & 0x01 {
            0 => Self::White,
            _ => Self::Yellow,
        }
    }
}

/// Letterboard WY (White/Yellow)
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LetterboardWY {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Colour
    pub colour: LetterboardWYColour,
    /// Heading / Direction
    pub heading: Heading,
    /// Mapping (6 bits, 0-63)
    pub character: Character,
    /// Floating
    pub floating: bool,
}

impl LetterboardWY {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = self.colour as u8 & 0x01;
        flags |= (self.character as u8 & 0x3f) << 1;
        if self.floating {
            flags |= 0x80;
        }
        ObjectFlags(flags)
    }

    pub(super) fn new(xyz: ObjectCoordinate, wire: ObjectFlags, heading: Heading) -> Result<Self, crate::DecodeError> {
        let colour = LetterboardWYColour::from(wire.0);
        let mapping = (wire.0 >> 1) & 0x3f;
        let character = Character::try_from(mapping)?;
        let floating = wire.floating();
        Ok(Self {
            xyz,
            colour,
            heading,
            character,
            floating,
        })
    }
}
