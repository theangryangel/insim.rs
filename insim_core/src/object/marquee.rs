//! Marquee objects
use super::{ObjectVariant, ObjectIntermediate};
use crate::heading::Heading;

/// Marquee
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Marquee {
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Marquee {
    fn to_wire(&self) -> Result<ObjectIntermediate, crate::EncodeError> {
        let mut flags = self.colour & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectIntermediate {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    fn from_wire(wire: ObjectIntermediate) -> Result<Self, crate::DecodeError> {
        let colour = wire.colour();
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
    fn test_marquee_round_trip() {
        let original = Marquee::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = Marquee::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
