//! Chalk ahead object
use super::{ObjectVariant, ObjectWire};
use crate::{DecodeError, heading::Heading};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Chalk Colour
pub enum ChalkColour {
    #[default]
    White,
    Red,
    Blue,
    Yellow,
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

impl From<ChalkColour> for u8 {
    fn from(colour: ChalkColour) -> Self {
        match colour {
            ChalkColour::White => 0,
            ChalkColour::Red => 1,
            ChalkColour::Blue => 2,
            ChalkColour::Yellow => 3,
        }
    }
}

/// Chalk ahead
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Chalk {
    /// Colour
    pub colour: ChalkColour,
    /// Heading / Direction
    pub heading: Heading,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Chalk {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = u8::from(self.colour) & 0x07;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError> {
        let colour = ChalkColour::from(wire.colour());
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
    fn test_chalk_ahead_round_trip() {
        let original = Chalk::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = Chalk::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
