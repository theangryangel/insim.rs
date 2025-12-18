//! Kerb objects
use super::{ObjectVariant, ObjectWire};
use crate::direction::Direction;

/// Kerb
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Kerb {
    /// Heading / Direction
    pub heading: Direction,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Kerb {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let index = 132;
        let mut flags = self.colour & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
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
