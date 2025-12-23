//! Post objects
use super::{ObjectVariant, ObjectWire};
use crate::direction::Heading;

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
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: PostColour,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Post {
    fn to_wire(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = self.colour as u8 & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    fn from_wire(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let colour = PostColour::from(wire.colour());
        let mapping = wire.mapping();
        let floating = wire.floating();
        Ok(Self {
            heading: Heading::from_objectinfo_wire(wire.heading),
            colour,
            mapping,
            floating,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_round_trip() {
        let original = Post::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = Post::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
