//! AutoX

use super::{ObjectCodec, ObjectPosition};

/// Tyre stack
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AutoX {
    /// XYZ position
    pub xyz: ObjectPosition,
    /// Position index
    pub position: u8,
    /// Heading / Direction
    pub heading: u8,
    /// Floating?
    pub floating: bool,
}

impl ObjectCodec for AutoX {
    fn encode(&self) -> Result<(&ObjectPosition, u8, u8), crate::EncodeError> {
        let mut flags = 0;
        flags |= self.position & 0x3f;
        if self.floating {
            flags |= 0x80;
        }
        Ok((&self.xyz, flags, self.heading))
    }

    fn decode(xyz: ObjectPosition, flags: u8, heading: u8) -> Result<Self, crate::DecodeError> {
        let position = flags & 0x3f;
        let floating = flags & 0x80 != 0;
        Ok(Self {
            xyz,
            position,
            heading,
            floating,
        })
    }
}
