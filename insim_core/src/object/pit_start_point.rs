//! Pit start point object
use crate::{
    heading::Heading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

/// Pit start point
/// Start Position
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let heading = Heading::from_objectinfo_wire(raw.heading);
        let pos_index = raw.flags & 0x3f;
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            heading,
            index: pos_index,
            floating,
        })
    }
}
impl ObjectInfoInner for PitStartPoint {
    fn flags(&self) -> u8 {
        let mut flags = self.index & 0x3f;
        if self.floating {
            flags |= 0x80;
        }
        flags
    }

    fn heading_mut(&mut self) -> Option<&mut Heading> {
        Some(&mut self.heading)
    }

    fn heading(&self) -> Option<Heading> {
        Some(self.heading)
    }

    fn floating(&self) -> Option<bool> {
        Some(self.floating)
    }

    fn heading_objectinfo_wire(&self) -> u8 {
        self.heading.to_objectinfo_wire()
    }
}
