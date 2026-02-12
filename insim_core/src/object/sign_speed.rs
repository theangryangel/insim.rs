//! Speed sign objects
use crate::{
    DecodeError, DecodeErrorKind,
    heading::Heading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
            found => Err(DecodeErrorKind::NoVariantMatch {
                found: found as u64,
            }
            .into()),
        }
    }
}

/// Speed Sign
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignSpeed {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Mapping
    pub mapping: SpeedSignMapping,
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Floating
    pub floating: bool,
}

impl SignSpeed {
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let heading = Heading::from_objectinfo_wire(raw.heading);
        let mapping = SpeedSignMapping::try_from(raw.raw_mapping())?;
        let colour = raw.raw_colour();
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            mapping,
            heading,
            colour,
            floating,
        })
    }
}
impl ObjectInfoInner for SignSpeed {
    fn flags(&self) -> u8 {
        let mut flags = self.colour & 0x07;
        flags |= (self.mapping as u8 & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        flags
    }

    fn heading_mut(&mut self) -> Option<&mut Heading> {
        Some(&mut self.heading)
    }

    fn heading(&self) -> Option<Heading> {
        Some(self.heading)
    }

    fn floating(&self) -> Option<bool> {
        Some(self.floating)
    }

    fn heading_objectinfo_wire(&self) -> u8 {
        self.heading.to_objectinfo_wire()
    }
}
