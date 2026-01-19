//! Control objects

use crate::{
    heading::Heading,
    object::{ObjectCoordinate, ObjectFlags},
};

#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Control object
pub struct Control {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Kind of Control Object
    pub kind: ControlKind,
    /// Heading
    pub heading: Heading,
    /// Floating?
    pub floating: bool,
}

impl Control {
    pub(super) fn to_flags(&self) -> ObjectFlags {
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

        ObjectFlags(flags)
    }

    pub(super) fn new(
        xyz: ObjectCoordinate,
        flags: ObjectFlags,
        heading: Heading,
    ) -> Result<Self, crate::DecodeError> {
        let position_bits = flags.0 & 0b11;
        let half_width = (flags.0 >> 2) & 0b11111;
        let floating = flags.floating();
        let kind = match position_bits {
            0b00 if half_width == 0 => ControlKind::Start,
            0b00 if half_width != 0 => ControlKind::Finish { half_width },
            0b01 => ControlKind::Checkpoint1 { half_width },
            0b10 => ControlKind::Checkpoint2 { half_width },
            0b11 => ControlKind::Checkpoint3 { half_width },
            _ => {
                return Err(crate::DecodeErrorKind::NoVariantMatch {
                    found: position_bits as u64,
                }.into());
            },
        };

        Ok(Self {
            xyz,
            kind,
            heading,
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
