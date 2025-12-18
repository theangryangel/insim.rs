//! Control objects
use super::{ObjectVariant, ObjectWire};
use crate::{DecodeError, direction::Direction};

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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Chalk Kind
pub enum ChalkKind {
    #[default]
    Line = 4,
    Line2,
    Ahead,
    Ahead2,
    Left,
    Left2,
    Left3,
    Right,
    Right2,
    Right3,
}

impl TryFrom<u8> for ChalkKind {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            4 => Ok(Self::Line),
            5 => Ok(Self::Line2),
            6 => Ok(Self::Ahead),
            7 => Ok(Self::Ahead2),
            8 => Ok(Self::Left),
            9 => Ok(Self::Left2),
            10 => Ok(Self::Left3),
            11 => Ok(Self::Right),
            12 => Ok(Self::Right2),
            13 => Ok(Self::Right3),
            found => Err(DecodeError::NoVariantMatch {
                found: found as u64,
            }),
        }
    }
}

/// Tyre stack
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Chalk {
    /// Kind of chalk
    pub kind: ChalkKind,
    /// Colour
    pub colour: ChalkColour,
    /// Heading / Direction
    pub heading: Direction,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Chalk {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let index = self.kind as u8;
        let mut flags = 0;
        flags |= self.colour as u8 & 0x07;
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
        let kind = ChalkKind::try_from(wire.index)?;
        let colour = ChalkColour::from(wire.colour());
        let floating = wire.floating();
        Ok(Self {
            kind,
            colour,
            heading: Direction::from_objectinfo_heading(wire.heading),
            floating,
        })
    }
}
