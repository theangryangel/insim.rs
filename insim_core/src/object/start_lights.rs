//! StartLights1 object
use crate::{
    DecodeError,
    heading::Heading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

/// StartLights
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StartLights {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Heading / Direction
    pub heading: Heading,
    /// identifier
    pub identifier: u8,
    /// Floating
    pub floating: bool,
}

impl StartLights {
    pub(super) fn new(raw: Raw) -> Result<Self, DecodeError> {
        let xyz = raw.xyz;
        let heading = Heading::from_objectinfo_wire(raw.heading);
        let identifier = raw.flags & 0x3F;
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            heading,
            identifier,
            floating,
        })
    }
}
impl ObjectInfoInner for StartLights {
    fn flags(&self) -> u8 {
        let mut flags = self.identifier & 0x3F;
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
