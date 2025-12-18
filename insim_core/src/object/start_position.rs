//! Start Position objects
use super::ObjectVariant;
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
    fn encode(&self) -> Result<(u8, u8, u8), crate::EncodeError> {
        let index = 184;
        let mut flags = self.index & 0x3f;
        if self.floating {
            flags |= 0x80;
        }
        let heading = self.heading.to_objectinfo_heading();
        Ok((index, flags, heading))
    }

    fn decode(_index: u8, flags: u8, heading: u8) -> Result<Self, crate::DecodeError> {
        let pos_index = flags & 0x3f;
        let floating = flags & 0x80 != 0;
        Ok(Self {
            heading: Direction::from_objectinfo_heading(heading),
            index: pos_index,
            floating,
        })
    }
}
