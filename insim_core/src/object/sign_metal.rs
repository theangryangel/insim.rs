//! Metal sign objects
use super::{ObjectVariant, ObjectWire};
use crate::{DecodeError, direction::Heading};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Metal Sign Mapping
pub enum MetalSignKind {
    /// Keep Left
    #[default]
    KeepLeft = 0,
    /// Keep Right
    KeepRight = 1,
    /// Left
    Left = 2,
    /// Right
    Right = 3,
    /// Up Left
    UpLeft = 4,
    /// Up Right
    UpRight = 5,
    /// Forward
    Forward = 6,
    /// No Entry
    NoEntry = 7,
}

impl TryFrom<u8> for MetalSignKind {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value & 0x0f {
            0 => Ok(Self::KeepLeft),
            1 => Ok(Self::KeepRight),
            2 => Ok(Self::Left),
            3 => Ok(Self::Right),
            4 => Ok(Self::UpLeft),
            5 => Ok(Self::UpRight),
            6 => Ok(Self::Forward),
            7 => Ok(Self::NoEntry),
            found => Err(DecodeError::NoVariantMatch {
                found: found as u64,
            }),
        }
    }
}

/// Metal Sign
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SignMetal {
    /// Kind
    pub kind: MetalSignKind,
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for SignMetal {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = self.colour & 0x07;
        flags |= (self.kind as u8 & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let kind = MetalSignKind::try_from(wire.mapping())?;
        let colour = wire.colour();
        let floating = wire.floating();
        Ok(Self {
            kind,
            heading: Heading::from_objectinfo_wire(wire.heading),
            colour,
            floating,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_metal_round_trip() {
        let original = SignMetal::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = SignMetal::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
