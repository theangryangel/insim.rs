//! Vehicle objects
use super::ObjectVariant;
use crate::{DecodeError, direction::Direction};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Vehicle Kind
pub enum VehicleKind {
    #[default]
    SUV = 124,
    Van = 125,
    Truck = 126,
    Ambulance = 127,
}

impl TryFrom<u8> for VehicleKind {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            124 => Ok(Self::SUV),
            125 => Ok(Self::Van),
            126 => Ok(Self::Truck),
            127 => Ok(Self::Ambulance),
            found => Err(DecodeError::NoVariantMatch {
                found: found as u64,
            }),
        }
    }
}

/// Vehicle
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Vehicle {
    /// Kind of vehicle
    pub kind: VehicleKind,
    /// Heading / Direction
    pub heading: Direction,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Vehicle {
    fn encode(&self) -> Result<(u8, u8, u8), crate::EncodeError> {
        let index = self.kind as u8;
        let mut flags = self.colour & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        let heading = self.heading.to_objectinfo_heading();
        Ok((index, flags, heading))
    }

    fn decode(index: u8, flags: u8, heading: u8) -> Result<Self, crate::DecodeError> {
        let kind = VehicleKind::try_from(index)?;
        let colour = flags & 0x07;
        let mapping = (flags >> 3) & 0x0f;
        let floating = flags & 0x80 != 0;
        Ok(Self {
            kind,
            heading: Direction::from_objectinfo_heading(heading),
            colour,
            mapping,
            floating,
        })
    }
}
