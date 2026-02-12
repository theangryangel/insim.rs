//! Insim objects
use crate::{
    heading::Heading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Insim Checkpoint Kind
pub enum InsimCheckpointKind {
    #[default]
    /// Finish line
    Finish = 0,
    /// Checkpoint 1
    Checkpoint1 = 1,
    /// Checkpoint 2
    Checkpoint2 = 2,
    /// Checkpoint 3
    Checkpoint3 = 3,
}

impl TryFrom<u8> for InsimCheckpointKind {
    type Error = crate::DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value & 0x03 {
            0 => Ok(Self::Finish),
            1 => Ok(Self::Checkpoint1),
            2 => Ok(Self::Checkpoint2),
            3 => Ok(Self::Checkpoint3),
            _ => unreachable!(),
        }
    }
}

/// InsimCheckpoint
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InsimCheckpoint {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Kind of checkpoint
    pub kind: InsimCheckpointKind,
    /// Heading / Direction
    pub heading: Heading,
    /// Floating
    pub floating: bool,
}

impl InsimCheckpoint {
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let heading = Heading::from_objectinfo_wire(raw.heading);
        let kind = InsimCheckpointKind::try_from(raw.flags)?;
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            kind,
            heading,
            floating,
        })
    }
}
impl ObjectInfoInner for InsimCheckpoint {
    fn flags(&self) -> u8 {
        let mut flags = 0;
        flags |= self.kind as u8;
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

/// Insim Circle
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InsimCircle {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Circle index (stored in heading byte on wire)
    pub index: u8,
    /// Floating
    pub floating: bool,
}

impl InsimCircle {
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let index = raw.heading;
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            index,
            floating,
        })
    }
}
impl ObjectInfoInner for InsimCircle {
    fn flags(&self) -> u8 {
        let mut flags = 0;
        if self.floating {
            flags |= 0x80;
        }
        flags
    }

    fn floating(&self) -> Option<bool> {
        Some(self.floating)
    }

    fn heading_objectinfo_wire(&self) -> u8 {
        self.index
    }
}
