use crate::object::marshal::MarshalKind;

use super::ObjectVariant;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum InsimCheckpointKind {
    #[default]
    Finish = 0,
    Checkpoint1 = 1,
    Checkpoint2 = 2,
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
    pub heading: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for InsimCheckpoint {
    fn encode(&self) -> Result<(u8, u8), crate::EncodeError> {
        let mut flags = 0;
        flags |= self.kind as u8;
        if self.floating {
            flags |= 0x80;
        }
        Ok((flags, self.heading))
    }

    fn decode(flags: u8, heading: u8) -> Result<Self, crate::DecodeError> {
        let kind = MarshalKind::try_from(flags)?;
        let floating = flags & 0x80 != 0;
        Ok(Self {
            kind,
            heading,
            floating,
        })
    }
}


/// InsimCheckpoint
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InsimCircle {
    /// Index
    pub index: u8,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for InsimCircle {
    fn encode(&self) -> Result<(u8, u8), crate::EncodeError> {
        let mut flags = 0;
        if self.floating {
            flags |= 0x80;
        }
        Ok((flags, self.index))
    }

    fn decode(flags: u8, heading: u8) -> Result<Self, crate::DecodeError> {
        let floating = flags & 0x80 != 0;
        Ok(Self {
            index: heading,
            floating,
        })
    }
}

