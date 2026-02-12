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
/// Chevron Colour
pub enum ChevronColour {
    /// White
    #[default]
    White = 0,
    /// Black
    Black = 1,
}

impl From<u8> for ChevronColour {
    fn from(value: u8) -> Self {
        match value & 0x07 {
            0 => Self::White,
            1 => Self::Black,
            _ => Self::White,
        }
    }
}

/// Chevron
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Chevron {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Colour
    pub colour: ChevronColour,
    /// Heading / Direction
    pub heading: Heading,
    /// Floating
    pub floating: bool,
}

impl Chevron {
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let heading = Heading::from_objectinfo_wire(raw.heading);
        let colour = ChevronColour::from(raw.raw_colour());
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            colour,
            heading,
            floating,
        })
    }
}
impl ObjectInfoInner for Chevron {
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
