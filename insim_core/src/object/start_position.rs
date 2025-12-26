//! Start Position objects
use super::{ObjectVariant, ObjectWire};
use crate::heading::Heading;

/// Start Position
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StartPosition {
    /// Heading / Direction
    pub heading: Heading,
    /// Position index (0-47, representing start positions 1-48)
    pub index: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for StartPosition {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = self.index & 0x3f;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let pos_index = wire.flags & 0x3f;
        let floating = wire.floating();
        Ok(Self {
            heading: Heading::from_objectinfo_wire(wire.heading),
            index: pos_index,
            floating,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_position_round_trip() {
        let original = StartPosition::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = StartPosition::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
