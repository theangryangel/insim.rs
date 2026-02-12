//! Post objects
use crate::{
    heading::Heading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
#[allow(missing_docs)]
/// Post Colour
pub enum PostColour {
    #[default]
    Green,
    Orange,
    Red,
    White,
    Blue,
    Yellow,
    LightBlue,
}

impl From<u8> for PostColour {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Green,
            1 => Self::Orange,
            2 => Self::Red,
            3 => Self::White,
            4 => Self::Blue,
            5 => Self::Yellow,
            6 => Self::LightBlue,
            _ => Self::Green,
        }
    }
}

/// Post
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Post {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: PostColour,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl Post {
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let heading = Heading::from_objectinfo_wire(raw.heading);
        let colour = PostColour::from(raw.raw_colour());
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
impl ObjectInfoInner for Post {
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
