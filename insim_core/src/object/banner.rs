//! Banner objects
use super::{ObjectVariant, ObjectIntermediate};
use crate::heading::Heading;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Chalk Colour
pub enum BannerColour {
    #[default]
    White = 0,
    Red,
    Yellow,
    Green,
    Blue,
    Black,
}

impl From<u8> for BannerColour {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::White,
            1 => Self::Red,
            2 => Self::Yellow,
            3 => Self::Green,
            4 => Self::Blue,
            5 => Self::Black,
            _ => Self::White,
        }
    }
}

/// Banner
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Banner {
    /// Heading / Direction
    pub heading: Heading,
    /// Colour (3 bits, 0-7)
    pub colour: BannerColour,
    /// Mapping (4 bits, 0-15)
    pub mapping: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Banner {
    fn to_wire(&self) -> Result<ObjectIntermediate, crate::EncodeError> {
        let mut flags = self.colour as u8 & 0x07;
        flags |= (self.mapping & 0x0f) << 3;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectIntermediate {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    fn from_wire(wire: ObjectIntermediate) -> Result<Self, crate::DecodeError> {
        let colour = BannerColour::from(wire.colour());
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
    fn test_banner_round_trip() {
        let original = Banner::default();
        let wire = original.to_wire().expect("to_wire failed");
        let decoded = Banner::from_wire(wire).expect("from_wire failed");
        assert_eq!(original, decoded);
    }
}
