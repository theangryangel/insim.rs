//! Letterboard objects
use super::{ObjectVariant, ObjectWire};
use crate::{DecodeError, direction::Direction};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Letterboard Colour
pub enum LetterboardColour {
    #[default]
    White = 0,
    Yellow = 1,
}

impl From<u8> for LetterboardColour {
    fn from(value: u8) -> Self {
        match value & 0x01 {
            0 => Self::White,
            _ => Self::Yellow,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Letterboard Kind
pub enum LetterboardKind {
    #[default]
    WY = 92,
    RB = 93,
}

impl TryFrom<u8> for LetterboardKind {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            92 => Ok(Self::WY),
            93 => Ok(Self::RB),
            found => Err(DecodeError::NoVariantMatch {
                found: found as u64,
            }),
        }
    }
}

/// Letterboard
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Letterboard {
    /// Kind of letterboard
    pub kind: LetterboardKind,
    /// Colour
    pub colour: LetterboardColour,
    /// Heading / Direction
    pub heading: Direction,
    /// Mapping (6 bits, 0-63)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Letterboard {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let index = self.kind as u8;
        let mut flags = self.colour as u8 & 0x01;
        flags |= (self.mapping & 0x3f) << 1;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            index,
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let kind = LetterboardKind::try_from(wire.index)?;
        let colour = LetterboardColour::from(wire.flags);
        let mapping = (wire.flags >> 1) & 0x3f;
        let floating = wire.floating();
        Ok(Self {
            kind,
            colour,
            heading: Direction::from_objectinfo_heading(wire.heading),
            mapping,
            floating,
        })
    }
}
