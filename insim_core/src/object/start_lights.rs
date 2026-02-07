//! StartLights1 object
use crate::{
    DecodeError,
    heading::Heading,
    object::{ObjectCoordinate, ObjectFlags},
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
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = self.identifier & 0x3F;
        if self.floating {
            flags |= 0x80;
        }
        ObjectFlags(flags)
    }

    pub(super) fn new(
        xyz: ObjectCoordinate,
        wire: ObjectFlags,
        heading: Heading,
    ) -> Result<Self, DecodeError> {
        let identifier = wire.0 & 0x3F;
        let floating = wire.floating();
        Ok(Self {
            xyz,
            heading,
            identifier,
            floating,
        })
    }
}
