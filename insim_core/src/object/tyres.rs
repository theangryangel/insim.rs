//! Tyre single object
use super::{ObjectVariant, ObjectIntermediate};
use crate::{DecodeError, heading::Heading};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
/// Tyre Stack Colour
pub enum TyreColour {
    /// Black
    #[default]
    Black = 0,
    /// White
    White = 1,
    /// Red
    Red = 2,
    /// Blue
    Blue = 3,
    /// Green
    Green = 4,
    /// Yellow
    Yellow = 5,
}

impl From<u8> for TyreColour {
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

/// Tyre single
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Tyres {
    /// Colour
    pub colour: TyreColour,
    /// Heading / Direction
    pub heading: Heading,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Tyres {
    fn to_wire(&self) -> Result<ObjectIntermediate, crate::EncodeError> {
        let mut flags = self.colour as u8 & 0x07;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectIntermediate {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    fn from_wire(wire: ObjectIntermediate) -> Result<Self, DecodeError> {
        let colour = TyreColour::from(wire.colour());
        let floating = wire.floating();
        Ok(Self {
            colour,
            heading: Heading::from_objectinfo_wire(wire.heading),
            floating,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tyre_single_round_trip() {
        let original = Tyres::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = Tyres::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
