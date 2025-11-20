//! Control objects

use super::{ObjectCodec, ObjectPosition};

/// Start position
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Point {
    /// XYZ position
    pub xyz: ObjectPosition,
    /// Heading
    pub heading: u8,
    /// Floating
    pub floating: bool,
}

/// Finish or Checkpoint position
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Gate {
    /// XYZ position
    pub xyz: ObjectPosition,
    /// Half width
    pub half_width: u8,
    /// Heading / Direction
    pub heading: u8,
    /// Floating
    pub floating: bool,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Marshall {
    #[default]
    None = 0,
    Standing = 1,
    Left = 2,
    Right = 3,
}

impl TryFrom<u8> for Marshall {
    type Error = crate::DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Marshall::None),
            1 => Ok(Marshall::Standing),
            2 => Ok(Marshall::Left),
            3 => Ok(Marshall::Right),
            // Catch-all for invalid values
            _ => Err(crate::DecodeError::NoVariantMatch {
                found: value as u64,
            }),
        }
    }
}

/// Marshall Circle / Restricted Area
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MarshallCircle {
    /// XYZ position
    pub xyz: ObjectPosition,
    /// Marshall
    pub marshall: Marshall,
    /// Radius
    pub radius: u8,
    /// Heading / Direction
    pub heading: u8,
    /// floating
    pub floating: bool,
}

impl ObjectCodec for MarshallCircle {
    fn encode(&self) -> Result<(&ObjectPosition, u8, u8), crate::EncodeError> {
        let mut flags = 0;
        flags |= self.marshall as u8;
        flags |= self.radius << 2;
        if self.floating {
            flags |= 0x80;
        }
        Ok((&self.xyz, flags, self.heading))
    }

    fn decode(xyz: ObjectPosition, flags: u8, heading: u8) -> Result<Self, crate::DecodeError> {
        let marshall = Marshall::try_from(flags & 0b11)?;
        let radius = (flags >> 2) & 0b11111;
        let floating = flags & 0x80 != 0;
        Ok(Self {
            xyz,
            marshall,
            radius,
            heading,
            floating,
        })
    }
}

/// Route Check
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RouteCheck {
    /// XYZ position
    pub xyz: ObjectPosition,
    /// Radius
    pub radius: u8,
    /// Heading / Direction
    pub heading: u8,
    /// floating
    pub floating: bool,
}

impl ObjectCodec for RouteCheck {
    fn encode(&self) -> Result<(&ObjectPosition, u8, u8), crate::EncodeError> {
        let mut flags = 0;
        flags |= (self.radius << 2) & 0x1f;
        if self.floating {
            flags |= 0x80;
        }
        Ok((&self.xyz, flags, self.heading))
    }

    fn decode(xyz: ObjectPosition, flags: u8, heading: u8) -> Result<Self, crate::DecodeError> {
        let radius = (flags >> 2) & 0x1f;
        let floating = flags & 0x80 != 0;
        Ok(Self {
            xyz,
            radius,
            heading,
            floating,
        })
    }
}

/// InsimCheckpoint
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InsimCheckpoint {
    /// XYZ position
    pub xyz: ObjectPosition,
    /// Heading / Direction
    pub heading: u8,
    /// floating
    pub floating: bool,
}

/// InsimCheckpoint
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InsimCircle {
    /// XYZ position
    pub xyz: ObjectPosition,
    /// Flags
    pub flags: u8,
    /// Index
    pub index: u8,
}
