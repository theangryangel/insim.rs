//! Bale objects
use super::ObjectVariant;

/// Bale
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Bale {
    /// Heading / Direction
    pub heading: u8,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Bale {
    fn encode(&self) -> Result<(u8, u8, u8), crate::EncodeError> {
        let index = 144;
        let mut flags = self.colour & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        Ok((index, flags, self.heading))
    }

    fn decode(_index: u8, flags: u8, heading: u8) -> Result<Self, crate::DecodeError> {
        let colour = flags & 0x07;
        let mapping = (flags >> 3) & 0x0f;
        let floating = flags & 0x80 != 0;
        Ok(Self {
            heading,
            colour,
            mapping,
            floating,
        })
    }
}
