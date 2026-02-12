//! Chalk ahead object
use crate::{
    DecodeError,
    heading::Heading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Chalk Colour
pub enum ChalkColour {
    #[default]
    White,
    Red,
    Blue,
    Yellow,
}

impl From<u8> for ChalkColour {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::White,
            1 => Self::Red,
            2 => Self::Blue,
            3 => Self::Yellow,
            _ => Self::White,
        }
    }
}

impl From<ChalkColour> for u8 {
    fn from(colour: ChalkColour) -> Self {
        match colour {
            ChalkColour::White => 0,
            ChalkColour::Red => 1,
            ChalkColour::Blue => 2,
            ChalkColour::Yellow => 3,
        }
    }
}

/// Chalk ahead
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Chalk {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Colour
    pub colour: ChalkColour,
    /// Heading / Direction
    pub heading: Heading,
    /// Floating
    pub floating: bool,
}

impl Chalk {
    pub(super) fn new(raw: Raw) -> Result<Self, DecodeError> {
        let xyz = raw.xyz;
        let heading = Heading::from_objectinfo_wire(raw.heading);
        let colour = ChalkColour::from(raw.raw_colour());
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            colour,
            heading,
            floating,
        })
    }
}
impl ObjectInfoInner for Chalk {
    fn flags(&self) -> u8 {
        let mut flags = u8::from(self.colour) & 0x07;
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
