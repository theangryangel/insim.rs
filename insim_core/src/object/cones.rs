//! Cone1 objects
use crate::{
    heading::Heading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Cone Colour
pub enum ConeColour {
    /// Red
    #[default]
    Red = 0,
    /// Blue
    Blue = 1,
    /// Blue (variant)
    Blue2 = 2,
    /// Green
    Green = 3,
    /// Orange
    Orange = 4,
    /// White
    White = 5,
    /// Yellow
    Yellow = 6,
}

impl From<u8> for ConeColour {
    fn from(value: u8) -> Self {
        match value & 0x07 {
            0 => Self::Red,
            1 => Self::Blue,
            2 => Self::Blue2,
            3 => Self::Green,
            4 => Self::Orange,
            5 => Self::White,
            6 => Self::Yellow,
            _ => Self::Red,
        }
    }
}

/// Cone1
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cone {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Colour
    pub colour: ConeColour,
    /// Heading / Direction
    pub heading: Heading,
    /// Floating
    pub floating: bool,
}

impl Cone {
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let heading = Heading::from_objectinfo_wire(raw.heading);
        let colour = ConeColour::from(raw.raw_colour());
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            colour,
            heading,
            floating,
        })
    }
}
impl ObjectInfoInner for Cone {
    fn flags(&self) -> u8 {
        let mut flags = 0;
        flags |= self.colour as u8 & 0x07;
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
