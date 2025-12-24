//! Cone1 objects
use super::{ObjectVariant, ObjectWire};
use crate::heading::Heading;

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
    /// Colour
    pub colour: ChevronColour,
    /// Heading / Direction
    pub heading: Heading,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Chevron {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= self.colour as u8 & 0x07;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let colour = ChevronColour::from(wire.colour());
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
    fn test_cone1_round_trip() {
        let original = Chevron::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = Chevron::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
