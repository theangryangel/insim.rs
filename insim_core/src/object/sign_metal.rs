//! Metal sign objects
use crate::{
    DecodeError, DecodeErrorKind,
    heading::ObjectHeading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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
            found => Err(DecodeErrorKind::NoVariantMatch {
                found: found as u64,
            }
            .into()),
        }
    }
}

/// Metal Sign
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SignMetal {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Kind
    pub kind: MetalSignKind,
    /// ObjectHeading / Direction
    pub heading: ObjectHeading,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Floating
    pub floating: bool,
}

impl SignMetal {
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let heading = ObjectHeading::from_raw(raw.heading);
        let kind = MetalSignKind::try_from(raw.raw_mapping())?;
        let colour = raw.raw_colour();
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            kind,
            heading,
            colour,
            floating,
        })
    }
}
impl ObjectInfoInner for SignMetal {
    fn flags(&self) -> u8 {
        let mut flags = self.colour & 0x07;
        flags |= (self.kind as u8 & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        flags
    }

    fn heading_mut(&mut self) -> Option<&mut ObjectHeading> {
        Some(&mut self.heading)
    }

    fn heading(&self) -> Option<ObjectHeading> {
        Some(self.heading)
    }

    fn floating(&self) -> Option<bool> {
        Some(self.floating)
    }

    fn floating_mut(&mut self) -> Option<&mut bool> {
        Some(&mut self.floating)
    }

    fn heading_objectinfo_wire(&self) -> u8 {
        self.heading.to_raw()
    }
}
