//! Start Position objects
use super::{ObjectVariant, ObjectWire};
use crate::direction::Direction;

/// Start Position
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StartPosition {
    /// Heading / Direction
    pub heading: Direction,
    /// Position index (0-47, representing start positions 1-48)
    pub index: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for StartPosition {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let index = 184;
        let mut flags = self.index & 0x3f;
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
        let pos_index = wire.flags & 0x3f;
        let floating = wire.floating();
        Ok(Self {
            heading: Direction::from_objectinfo_heading(wire.heading),
            index: pos_index,
            floating,
        })
    }
}
