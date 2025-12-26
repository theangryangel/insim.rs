//! Speed sign objects
use super::{ObjectVariant, ObjectWire};
use crate::{DecodeError, heading::Heading};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Speed Sign Mapping
pub enum SpeedSignMapping {
    /// 80 km/h
    #[default]
    Speed80Kmh = 0,
    /// 50 km/h
    Speed50Kmh = 1,
    /// 50 mph
    Speed50Mph = 2,
    /// 40 mph
    Speed40Mph = 3,
}

impl TryFrom<u8> for SpeedSignMapping {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value & 0x0f {
            0 => Ok(Self::Speed80Kmh),
            1 => Ok(Self::Speed50Kmh),
            2 => Ok(Self::Speed50Mph),
            3 => Ok(Self::Speed40Mph),
            found => Err(DecodeError::NoVariantMatch {
                found: found as u64,
            }),
        }
    }
}

/// Speed Sign
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SignSpeed {
    /// Mapping
    pub mapping: SpeedSignMapping,
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for SignSpeed {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = self.colour & 0x07;
        flags |= (self.mapping as u8 & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let mapping = SpeedSignMapping::try_from(wire.mapping())?;
        let colour = wire.colour();
        let floating = wire.floating();
        Ok(Self {
            mapping,
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
    fn test_sign_speed_round_trip() {
        let original = SignSpeed::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = SignSpeed::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
