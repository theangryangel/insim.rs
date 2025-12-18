//! Sign objects
use super::{ObjectVariant, ObjectWire};
use crate::{DecodeError, direction::Direction};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Sign Kind
pub enum SignKind {
    #[default]
    Metal = 160,
    Speed = 168,
}

impl TryFrom<u8> for SignKind {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            160 => Ok(Self::Metal),
            168 => Ok(Self::Speed),
            found => Err(DecodeError::NoVariantMatch {
                found: found as u64,
            }),
        }
    }
}

/// Sign
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Sign {
    /// Kind of sign
    pub kind: SignKind,
    /// Heading / Direction
    pub heading: Direction,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Sign {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let index = self.kind as u8;
        let mut flags = self.colour & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            index,
            flags,
            heading: self.heading.to_objectinfo_heading(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let kind = SignKind::try_from(wire.index)?;
        let colour = wire.colour();
        let mapping = wire.mapping();
        let floating = wire.floating();
        Ok(Self {
            kind,
            heading: Direction::from_objectinfo_heading(wire.heading),
            colour,
            mapping,
            floating,
        })
    }
}
