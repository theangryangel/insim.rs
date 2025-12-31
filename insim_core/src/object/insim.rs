//! Insim objects
use crate::{
    heading::Heading,
    object::{ObjectCoordinate, ObjectFlags},
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = 0;
        flags |= self.kind as u8;
        if self.floating {
            flags |= 0x80;
        }
        ObjectFlags(flags)
    }

    pub(crate) fn new(
        xyz: ObjectCoordinate,
        wire: ObjectFlags,
        heading: Heading,
    ) -> Result<Self, crate::DecodeError> {
        let kind = InsimCheckpointKind::try_from(wire.0)?;
        let floating = wire.floating();
        Ok(Self {
            xyz,
            kind,
            heading,
            floating,
        })
    }
}

/// Insim Circle
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InsimCircle {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Circle index (stored in heading byte on wire)
    pub index: u8,
    /// Floating
    pub floating: bool,
}

impl InsimCircle {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = 0;
        if self.floating {
            flags |= 0x80;
        }
        ObjectFlags(flags)
    }

    pub(crate) fn new(
        xyz: ObjectCoordinate,
        wire: ObjectFlags,
        index: u8,
    ) -> Result<Self, crate::DecodeError> {
        let floating = wire.floating();
        Ok(Self {
            xyz,
            index,
            floating,
        })
    }
}
