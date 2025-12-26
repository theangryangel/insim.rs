//! Letterboard WY (White/Yellow) objects
use super::{ObjectVariant, ObjectWire, letterboard_rb::Character};
use crate::heading::Heading;

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
    /// Colour
    pub colour: LetterboardWYColour,
    /// Heading / Direction
    pub heading: Heading,
    /// Mapping (6 bits, 0-63)
    pub character: Character,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for LetterboardWY {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = self.colour as u8 & 0x01;
        flags |= (self.character as u8 & 0x3f) << 1;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let colour = LetterboardWYColour::from(wire.flags);
        let mapping = (wire.flags >> 1) & 0x3f;
        let character = Character::try_from(mapping)?;
        let floating = wire.floating();
        Ok(Self {
            colour,
            heading: Heading::from_objectinfo_wire(wire.heading),
            character,
            floating,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_letterboard_w_y_round_trip() {
        let original = LetterboardWY::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = LetterboardWY::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
