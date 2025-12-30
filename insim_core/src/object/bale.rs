//! Bale objects
use crate::{heading::Heading, object::{ObjectCoordinate, ObjectFlags}};

/// Bale
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Bale {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl Bale {
    pub(super) fn to_flags(&self) -> ObjectFlags { 
        let mut flags = self.colour & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        ObjectFlags(flags)
    }

    pub(super) fn new(xyz: ObjectCoordinate, flags: ObjectFlags, heading: Heading) -> Result<Self, crate::DecodeError> {
        let colour = flags.colour();
        let mapping = flags.mapping();
        let floating = flags.floating();
        Ok(Self {
            xyz,
            heading,
            colour,
            mapping,
            floating,
        })
    }
}
