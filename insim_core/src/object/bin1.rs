//! Bin1 object
use crate::{
    DecodeError,
    heading::ObjectHeading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
#[allow(missing_docs)]
/// Bin Colour
pub enum Bin1Colour {
    #[default]
    Red,
    Yellow,
    Blue,
    Green,
    White,
    Orange,
}

impl From<u8> for Bin1Colour {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Red,
            1 => Self::Yellow,
            2 => Self::Blue,
            3 => Self::Green,
            4 => Self::White,
            5 => Self::Orange,
            _ => Self::Green,
        }
    }
}

/// Bin1
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Bin1 {
    /// Position
    pub xyz: ObjectCoordinate,
    /// ObjectHeading / Direction
    pub heading: ObjectHeading,
    /// Colour (3 bits, 0-7)
    pub colour: Bin1Colour,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl Bin1 {
    pub(super) fn new(raw: Raw) -> Result<Self, DecodeError> {
        let xyz = raw.xyz;
        let heading = ObjectHeading::from_raw(raw.heading);
        let colour = Bin1Colour::from(raw.raw_colour());
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
impl ObjectInfoInner for Bin1 {
    fn flags(&self) -> u8 {
        let mut flags = self.colour as u8 & 0x07;
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
