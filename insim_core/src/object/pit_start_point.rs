//! Pit start point object
use crate::{heading::Heading, object::{ObjectCoordinate, ObjectFlags}};

/// Pit start point
/// Start Position
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PitStartPoint {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Heading / Direction
    pub heading: Heading,
    /// Position index (0-47, representing start positions 1-48)
    pub index: u8,
    /// Floating
    pub floating: bool,
}

impl PitStartPoint {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = self.index & 0x3f;
        if self.floating {
            flags |= 0x80;
        }
        ObjectFlags(flags)
    }

    pub(super) fn new(xyz: ObjectCoordinate, wire: ObjectFlags, heading: Heading) -> Result<Self, crate::DecodeError> {
        let pos_index = wire.0 & 0x3f;
        let floating = wire.floating();
        Ok(Self {
            xyz,
            heading,
            index: pos_index,
            floating,
        })
    }
}
