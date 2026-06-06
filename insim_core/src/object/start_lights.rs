//! StartLights1 object
use crate::{
    DecodeError,
    heading::ObjectHeading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

/// StartLights
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct StartLights {
    /// Position
    pub xyz: ObjectCoordinate,
    /// ObjectHeading / Direction
    pub heading: ObjectHeading,
    /// identifier
    pub identifier: u8,
    /// Floating
    pub floating: bool,
}

impl StartLights {
    pub(super) fn new(raw: Raw) -> Result<Self, DecodeError> {
        let xyz = raw.xyz;
        let heading = ObjectHeading::from_raw(raw.heading);
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
