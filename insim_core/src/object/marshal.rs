//! Marshal objects
use super::ObjectWire;
use crate::heading::Heading;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Marshal
pub struct Marshal {
    /// Kind of Marshal
    pub kind: MarshalKind,
    /// Heading
    pub heading: Heading,
    /// Floating?
    pub floating: bool,
}

impl Marshal {
    pub(crate) fn encode(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags: u8 = self.kind as u8;
        if self.floating {
            flags |= 0x80;
        }

        Ok(ObjectWire {
            flags,
            heading: self.heading.to_objectinfo_wire(),
        })
    }

    pub(crate) fn decode(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let kind = MarshalKind::try_from(wire.flags)?;
        let floating = wire.floating();

        Ok(Self {
            kind,
            heading: Heading::from_objectinfo_wire(wire.heading),
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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RestrictedArea {
    /// Radius
    pub radius: u8,
    /// floating
    pub floating: bool,
}

impl RestrictedArea {
    pub(crate) fn encode(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= self.radius << 2;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire { flags, heading: 0 })
    }

    pub(crate) fn decode(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let radius = (wire.flags >> 2) & 0b11111;
        let floating = wire.floating();
        Ok(Self { radius, floating })
    }
}

/// Route Check
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RouteChecker {
    /// Route index (stored in heading byte on wire)
    pub route: u8,
    /// Radius
    pub radius: u8,
    /// floating
    pub floating: bool,
}

impl RouteChecker {
    pub(crate) fn encode(&self) -> Result<ObjectWire, crate::EncodeError> {
        let mut flags = 0;
        flags |= (self.radius << 2) & 0b11111;
        if self.floating {
            flags |= 0x80;
        }
        Ok(ObjectWire {
            flags,
            heading: self.route,
        })
    }

    pub(crate) fn decode(wire: ObjectWire) -> Result<Self, crate::DecodeError> {
        let radius = (wire.flags >> 2) & 0b11111;
        let floating = wire.floating();
        Ok(Self {
            radius,
            floating,
            route: wire.heading,
        })
    }
}
