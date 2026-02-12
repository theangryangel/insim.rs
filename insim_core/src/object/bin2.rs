//! Bin2 object
use crate::{
    DecodeError,
    heading::Heading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
#[allow(missing_docs)]
/// Bin Colour
pub enum Bin2Colour {
    #[default]
    Green,
    Blue,
    Yellow,
    Black,
    White,
    Orange,
}

impl From<u8> for Bin2Colour {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Green,
            1 => Self::Blue,
            2 => Self::Yellow,
            3 => Self::Black,
            4 => Self::White,
            5 => Self::Orange,
            _ => Self::Green,
        }
    }
}

/// Bin2
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bin2 {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: Bin2Colour,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl Bin2 {
    pub(super) fn new(raw: Raw) -> Result<Self, DecodeError> {
        let xyz = raw.xyz;
        let heading = Heading::from_objectinfo_wire(raw.heading);
        let colour = Bin2Colour::from(raw.raw_colour());
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
impl ObjectInfoInner for Bin2 {
    fn flags(&self) -> u8 {
        let mut flags = self.colour as u8 & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
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
