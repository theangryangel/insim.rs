//! Letterboard WY (White/Yellow) objects
use super::letterboard_rb::Character;
use crate::{
    heading::Heading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let heading = Heading::from_objectinfo_wire(raw.heading);
        let colour = LetterboardWYColour::from(raw.flags);
        let mapping = (raw.flags >> 1) & 0x3f;
        let character = Character::try_from(mapping)?;
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            colour,
            heading,
            character,
            floating,
        })
    }
}
impl ObjectInfoInner for LetterboardWY {
    fn flags(&self) -> u8 {
        let mut flags = self.colour as u8 & 0x01;
        flags |= (self.character as u8 & 0x3f) << 1;
        if self.floating {
            flags |= 0x80;
        }
        flags
    }

    fn heading_mut(&mut self) -> Option<&mut Heading> {
        Some(&mut self.heading)
    }

    fn heading(&self) -> Option<Heading> {
        Some(self.heading)
    }

    fn floating(&self) -> Option<bool> {
        Some(self.floating)
    }

    fn heading_objectinfo_wire(&self) -> u8 {
        self.heading.to_objectinfo_wire()
    }
}
