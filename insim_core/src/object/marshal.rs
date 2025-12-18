//! Marshal objects
use crate::direction::Direction;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Marshal
pub struct Marshal {
    /// Kind of Marshal
    pub kind: MarshalKind,
    /// Heading
    pub heading: Direction,
    /// Floating?
    pub floating: bool,
}

impl Marshal {
    pub(crate) fn encode(&self) -> Result<(u8, u8), crate::EncodeError> {
        let mut flags: u8 = self.kind as u8;
        if self.floating {
            flags |= 0x80;
        }

        let heading = self.heading.to_objectinfo_heading();
        Ok((flags, heading))
    }

    pub(crate) fn decode(flags: u8, heading: u8) -> Result<Self, crate::DecodeError> {
        let kind = MarshalKind::try_from(flags)?;
        let floating = flags & 0x80 != 0;

        Ok(Self {
            kind,
            heading: Direction::from_objectinfo_heading(heading),
            floating,
        })
    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum MarshalKind {
    #[default]
    Standing = 1,
    Left = 2,
    Right = 3,
}

impl TryFrom<u8> for MarshalKind {
    type Error = crate::DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value & 0x02 {
            0 => Ok(MarshalKind::Standing),
            1 => Ok(MarshalKind::Left),
            2 => Ok(MarshalKind::Right),
            _ => unreachable!(),
        }
    }
}

/// Marshall Circle / Restricted Area
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RestrictedArea {
    /// Radius
    pub radius: u8,
    /// floating
    pub floating: bool,
}

impl RestrictedArea {
    pub(crate) fn encode(&self) -> Result<(u8, u8), crate::EncodeError> {
        let mut flags = 0;
        flags |= self.radius << 2;
        if self.floating {
            flags |= 0x80;
        }
        Ok((flags, 0))
    }

    pub(crate) fn decode(flags: u8, _heading: u8) -> Result<Self, crate::DecodeError> {
        let radius = (flags >> 2) & 0b11111;
        let floating = flags & 0x80 != 0;
        Ok(Self { radius, floating })
    }
}

/// Route Check
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RouteChecker {
    /// Index
    pub index: u8,
    /// Radius
    pub radius: u8,
    /// floating
    pub floating: bool,
}

impl RouteChecker {
    pub(crate) fn encode(&self) -> Result<(u8, u8), crate::EncodeError> {
        let mut flags = 0;
        flags |= (self.radius << 2) & 0b11111;
        if self.floating {
            flags |= 0x80;
        }
        Ok((flags, self.index))
    }

    pub(crate) fn decode(flags: u8, heading: u8) -> Result<Self, crate::DecodeError> {
        let radius = (flags >> 2) & 0b11111;
        let floating = flags & 0x80 != 0;
        Ok(Self {
            radius,
            floating,
            index: heading,
        })
    }
}
