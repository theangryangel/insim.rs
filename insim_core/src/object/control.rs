//! Control objects

use crate::{
    heading::Heading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let heading = Heading::from_objectinfo_wire(raw.heading);
        let position_bits = raw.flags & 0b11;
        let half_width = (raw.flags >> 2) & 0b11111;
        let floating = raw.raw_floating();
        let kind = match position_bits {
            0b00 if half_width == 0 => ControlKind::Start,
            0b00 if half_width != 0 => ControlKind::Finish { half_width },
            0b01 => ControlKind::Checkpoint1 { half_width },
            0b10 => ControlKind::Checkpoint2 { half_width },
            0b11 => ControlKind::Checkpoint3 { half_width },
            _ => {
                return Err(crate::DecodeErrorKind::NoVariantMatch {
                    found: position_bits as u64,
                }
                .into());
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
impl ObjectInfoInner for Control {
    fn flags(&self) -> u8 {
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

/// Control Kind
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
