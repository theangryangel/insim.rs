//! Marquee objects
use crate::{
    heading::ObjectHeading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

/// Marquee
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Marquee {
    /// Position
    pub xyz: ObjectCoordinate,
    /// ObjectHeading / Direction
    pub heading: ObjectHeading,
    /// Colour (3 bits, 0-7)
    pub colour: u8,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl Marquee {
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let heading = ObjectHeading::from_raw(raw.heading);
        let colour = raw.raw_colour();
        let mapping = raw.raw_mapping();
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            heading,
            colour,
            mapping,
            floating,
        })
    }
}
impl ObjectInfoInner for Marquee {
    fn flags(&self) -> u8 {
        let mut flags = self.colour & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
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
