//! Insim objects
use super::ObjectWire;
use crate::direction::Heading;

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
    /// Kind of checkpoint
    pub kind: InsimCheckpointKind,
    /// Heading / Direction
    pub heading: Heading,
    /// Floating
    pub floating: bool,
}

impl InsimCheckpoint {
    pub(crate) fn encode(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= self.kind as u8;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    pub(crate) fn decode(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let kind = InsimCheckpointKind::try_from(wire.flags)?;
        let floating = wire.floating();
        Ok(Self {
            kind,
            heading: Heading::from_objectinfo_wire(wire.heading),
            floating,
        })
    }
}

/// Insim Circle
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InsimCircle {
    /// Circle index (stored in heading byte on wire)
    pub index: u8,
    /// Floating
    pub floating: bool,
}

impl InsimCircle {
    pub(crate) fn encode(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.index,
        })
    }

    pub(crate) fn decode(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let floating = wire.floating();
        Ok(Self {
            index: wire.heading,
            floating,
        })
    }
}
