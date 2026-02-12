//! Tyre single object
use crate::{
    DecodeError,
    heading::Heading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
/// Tyre Stack Colour
pub enum TyreColour {
    /// Black
    #[default]
    Black = 0,
    /// White
    White = 1,
    /// Red
    Red = 2,
    /// Blue
    Blue = 3,
    /// Green
    Green = 4,
    /// Yellow
    Yellow = 5,
}

impl From<u8> for TyreColour {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Black,
            1 => Self::White,
            2 => Self::Red,
            3 => Self::Blue,
            4 => Self::Green,
            5 => Self::Yellow,
            _ => Self::Black,
        }
    }
}

/// Tyre single
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tyres {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Colour
    pub colour: TyreColour,
    /// Heading / Direction
    pub heading: Heading,
    /// Floating
    pub floating: bool,
}

impl Tyres {
    pub(super) fn new(raw: Raw) -> Result<Self, DecodeError> {
        let xyz = raw.xyz;
        let heading = Heading::from_objectinfo_wire(raw.heading);
        let colour = TyreColour::from(raw.raw_colour());
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            colour,
            heading,
            floating,
        })
    }
}
impl ObjectInfoInner for Tyres {
    fn flags(&self) -> u8 {
        let mut flags = self.colour as u8 & 0x07;
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
