//! Speed hump objects
use super::ObjectVariant;
use crate::DecodeError;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Speed Hump Kind
pub enum SpeedHumpKind {
    #[default]
    Hump10M = 128,
    Hump6M = 129,
    Hump2M = 130,
    Hump1M = 131,
}

impl TryFrom<u8> for SpeedHumpKind {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            128 => Ok(Self::Hump10M),
            129 => Ok(Self::Hump6M),
            130 => Ok(Self::Hump2M),
            131 => Ok(Self::Hump1M),
            found => Err(DecodeError::NoVariantMatch {
                found: found as u64,
            }),
        }
    }
}

/// Speed hump
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SpeedHump {
    /// Kind of speed hump
    pub kind: SpeedHumpKind,
    /// Heading / Direction
    pub heading: u8,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for SpeedHump {
    fn encode(&self) -> Result<(u8, u8, u8), crate::EncodeError> {
        let index = self.kind as u8;
        let mut flags = self.colour & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        Ok((index, flags, self.heading))
    }

    fn decode(index: u8, flags: u8, heading: u8) -> Result<Self, crate::DecodeError> {
        let kind = SpeedHumpKind::try_from(index)?;
        let colour = flags & 0x07;
        let mapping = (flags >> 3) & 0x0f;
        let floating = flags & 0x80 != 0;
        Ok(Self {
            kind,
            heading,
            colour,
            mapping,
            floating,
        })
    }
}
