//! Banner objects
use super::ObjectVariant;
use crate::direction::Direction;

/// Banner
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Banner {
    /// Heading / Direction
    pub heading: Direction,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Banner {
    fn encode(&self) -> Result<(u8, u8, u8), crate::EncodeError> {
        let index = 112;
        let mut flags = self.colour & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        let heading = self.heading.to_objectinfo_heading();
        Ok((index, flags, heading))
    }

    fn decode(_index: u8, flags: u8, heading: u8) -> Result<Self, crate::DecodeError> {
        let colour = flags & 0x07;
        let mapping = (flags >> 3) & 0x0f;
        let floating = flags & 0x80 != 0;
        Ok(Self {
            heading: Direction::from_objectinfo_heading(heading),
            colour,
            mapping,
            floating,
        })
    }
}
