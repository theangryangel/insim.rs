//! Marshal objects
use crate::{
    heading::Heading,
    object::{ObjectCoordinate, ObjectInfoInner, Raw},
};

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Marshal
pub struct Marshal {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Kind of Marshal
    pub kind: MarshalKind,
    /// Heading
    pub heading: Heading,
    /// Floating?
    pub floating: bool,
}

impl Marshal {
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let heading = Heading::from_objectinfo_wire(raw.heading);
        let kind = MarshalKind::try_from(raw.flags)?;
        let floating = raw.raw_floating();

        Ok(Self {
            xyz,
            kind,
            heading,
            floating,
        })
    }
}
impl ObjectInfoInner for Marshal {
    fn flags(&self) -> u8 {
        let mut flags: u8 = self.kind as u8;
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

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        match value & 0x03 {
            0 => Ok(MarshalKind::Standing),
            1 => Ok(MarshalKind::Left),
            2 => Ok(MarshalKind::Right),
            _ => unreachable!(),
        }
    }
}

/// Marshall Circle / Restricted Area
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RestrictedArea {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Radius
    pub radius: u8,
    /// floating
    pub floating: bool,
}

impl RestrictedArea {
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let radius = (raw.flags >> 2) & 0b11111;
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            radius,
            floating,
        })
    }
}
impl ObjectInfoInner for RestrictedArea {
    fn flags(&self) -> u8 {
        let mut flags = 0;
        flags |= self.radius << 2;
        if self.floating {
            flags |= 0x80;
        }
        flags
    }

    fn floating(&self) -> Option<bool> {
        Some(self.floating)
    }

    fn heading_objectinfo_wire(&self) -> u8 {
        self.radius
    }
}

/// Route Check
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RouteChecker {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Route index (stored in heading byte on wire)
    pub route: u8,
    /// Radius
    pub radius: u8,
    /// floating
    pub floating: bool,
}

impl RouteChecker {
    pub(super) fn new(raw: Raw) -> Result<Self, crate::DecodeError> {
        let xyz = raw.xyz;
        let route = raw.heading;
        let radius = (raw.flags >> 2) & 0b11111;
        let floating = raw.raw_floating();
        Ok(Self {
            xyz,
            radius,
            floating,
            route,
        })
    }
}
impl ObjectInfoInner for RouteChecker {
    fn flags(&self) -> u8 {
        let mut flags = 0;
        flags |= (self.radius << 2) & 0b11111;
        if self.floating {
            flags |= 0x80;
        }
        flags
    }

    fn floating(&self) -> Option<bool> {
        Some(self.floating)
    }

    fn heading_objectinfo_wire(&self) -> u8 {
        self.radius
    }
}
