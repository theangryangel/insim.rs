//! Bin1 object
use super::{ObjectVariant, ObjectWire};
use crate::{DecodeError, heading::Heading};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
#[allow(missing_docs)]
/// Post Colour
pub enum Bin1Colour {
    #[default]
    Red,
    Yellow,
    Blue,
    Green,
    White,
    Orange,
}

impl From<u8> for Bin1Colour {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Red,
            1 => Self::Yellow,
            2 => Self::Blue,
            3 => Self::Green,
            4 => Self::White,
            5 => Self::Orange,
            _ => Self::Green,
        }
    }
}

/// Bin1
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Bin1 {
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: Bin1Colour,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Bin1 {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = self.colour as u8 & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError> {
        let colour = Bin1Colour::from(wire.colour());
        let mapping = wire.mapping();
        let floating = wire.floating();
        Ok(Self {
            heading: Heading::from_objectinfo_wire(wire.heading),
            colour,
            mapping,
            floating,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bin1_round_trip() {
        let original = Bin1::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = Bin1::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
