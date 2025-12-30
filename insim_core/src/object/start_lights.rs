//! StartLights1 object
use super::{ObjectVariant, ObjectIntermediate};
use crate::{DecodeError, heading::Heading};

/// StartLights
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StartLights {
    /// Heading / Direction
    pub heading: Heading,
    /// identifier
    pub identifier: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for StartLights {
    fn to_wire(&self) -> Result<ObjectIntermediate, crate::EncodeError> {
        let mut flags = self.identifier & 0x3F;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectIntermediate {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    fn from_wire(wire: ObjectIntermediate) -> Result<Self, DecodeError> {
        let identifier = wire.flags & 0x3F;
        let floating = wire.floating();
        Ok(Self {
            heading: Heading::from_objectinfo_wire(wire.heading),
            identifier,
            floating,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_lights_round_trip() {
        let original = StartLights::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = StartLights::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
