//! Ramp1 object
use super::{ObjectVariant, ObjectWire};
use crate::{DecodeError, direction::Direction};

/// Ramp1
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Ramp {
    /// Heading / Direction
    pub heading: Direction,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Ramp {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = self.colour & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, DecodeError> {
        let colour = wire.colour();
        let mapping = wire.mapping();
        let floating = wire.floating();
        Ok(Self {
            heading: Direction::from_objectinfo_heading(wire.heading),
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
    fn test_ramp1_round_trip() {
        let original = Ramp::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = Ramp::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
