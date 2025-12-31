//! Post objects
use crate::{
    heading::Heading,
    object::{ObjectCoordinate, ObjectFlags},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = self.colour as u8 & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        ObjectFlags(flags)
    }

    pub(super) fn new(
        xyz: ObjectCoordinate,
        wire: ObjectFlags,
        heading: Heading,
    ) -> Result<Self, crate::DecodeError> {
        let colour = PostColour::from(wire.colour());
        let mapping = wire.mapping();
        let floating = wire.floating();
        Ok(Self {
            xyz,
            heading,
            colour,
            mapping,
            floating,
        })
    }
}
