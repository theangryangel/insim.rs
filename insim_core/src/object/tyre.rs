//! Control objects
use super::{ObjectVariant, ObjectWire};
use crate::{DecodeError, direction::Direction};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Tyre Kind
pub enum TyreStackKind {
    #[default]
    Single = 46,
    Stack2,
    Stack3,
    Stack4,
    SingleBig,
    Stack2Big,
    Stack3Big,
    Stack4Big,
}

impl TryFrom<u8> for TyreStackKind {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            46 => Ok(Self::Single),
            47 => Ok(Self::Stack2),
            48 => Ok(Self::Stack3),
            49 => Ok(Self::Stack4),
            50 => Ok(Self::SingleBig),
            51 => Ok(Self::Stack2Big),
            52 => Ok(Self::Stack3Big),
            found => Err(DecodeError::NoVariantMatch {
                found: found as u64,
            }),
        }
    }
}

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
    /// Kind
    pub kind: TyreStackKind,
    /// Colour
    pub colour: TyreStackColour,
    /// Heading / Direction
    pub heading: Direction,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for TyreStack {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= self.colour as u8 & 0x07;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            index: self.kind as u8,
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let kind = TyreStackKind::try_from(wire.index)?;
        let colour = TyreStackColour::from(wire.colour());
        let floating = wire.floating();
        Ok(Self {
            kind,
            colour,
            heading: Direction::from_objectinfo_heading(wire.heading),
            floating,
        })
    }
}
