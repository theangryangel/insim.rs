//! Control objects

use super::ObjectWire;
use crate::heading::Heading;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Control object
pub struct Control {
    /// Kind of Control Object
    pub kind: ControlKind,
    /// Heading
    pub heading: Heading,
    /// Floating?
    pub floating: bool,
}

impl Control {
    pub(crate) fn encode(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = match self.kind {
            ControlKind::Start => 0,
            ControlKind::Checkpoint1 { half_width } => (half_width << 2) | 0b01,
            ControlKind::Checkpoint2 { half_width } => (half_width << 2) | 0b10,
            ControlKind::Checkpoint3 { half_width } => (half_width << 2) | 0b11,
            ControlKind::Finish { half_width } => half_width << 2,
        };
        if self.floating {
            flags |= 0x80;
        }

        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    pub(crate) fn decode(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let position_bits = wire.flags & 0b11;
        let half_width = (wire.flags >> 2) & 0b11111;
        let floating = wire.floating();
        let kind = match position_bits {
            0b00 if half_width == 0 => ControlKind::Start,
            0b00 if half_width != 0 => ControlKind::Finish { half_width },
            0b01 => ControlKind::Checkpoint1 { half_width },
            0b10 => ControlKind::Checkpoint2 { half_width },
            0b11 => ControlKind::Checkpoint3 { half_width },
            _ => {
                return Err(crate::DecodeError::NoVariantMatch {
                    found: position_bits as u64,
                });
            },
        };

        Ok(Self {
            kind,
            heading: Heading::from_objectinfo_wire(wire.heading),
            floating,
        })
    }
}

/// Control Kind
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Kind of Control Object
pub enum ControlKind {
    #[default]
    /// Start line
    Start,
    /// Checkpoint 1
    Checkpoint1 {
        /// Half width
        half_width: u8,
    },
    /// Checkpoint 2
    Checkpoint2 {
        /// Half width
        half_width: u8,
    },
    /// Checkpoint 3
    Checkpoint3 {
        /// Half width
        half_width: u8,
    },
    /// Finish line
    Finish {
        /// Half width
        half_width: u8,
    },
}
