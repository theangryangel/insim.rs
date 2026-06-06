//! Pit start point object
use crate::{
    heading::ObjectHeading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

/// Pit start point
/// Start Position
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PitStartPoint {
    /// Position
    pub xyz: ObjectCoordinate,
    /// ObjectHeading / Direction
    pub heading: ObjectHeading,
    /// Position index (0-47, representing start positions 1-48)
    pub index: u8,
    /// Floating
    pub floating: bool,
}

impl PitStartPoint {
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let heading = ObjectHeading::from_raw(raw.heading);
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

    fn heading_mut(&mut self) -> Option<&mut ObjectHeading> {
        Some(&mut self.heading)
    }

    fn heading(&self) -> Option<ObjectHeading> {
        Some(self.heading)
    }

    fn floating(&self) -> Option<bool> {
        Some(self.floating)
    }

    fn floating_mut(&mut self) -> Option<&mut bool> {
        Some(&mut self.floating)
    }

    fn heading_objectinfo_wire(&self) -> u8 {
        self.heading.to_raw()
    }
}
