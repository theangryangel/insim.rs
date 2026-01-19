//! Marker objects
use crate::{
    DecodeError, DecodeErrorKind,
    heading::Heading,
    object::{ObjectCoordinate, ObjectFlags},
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Marker Corner Kind
pub enum MarkerCornerKind {
    #[default]
    CurveL = 0,
    CurveR = 1,
    L = 2,
    R = 3,
    HardL = 4,
    HardR = 5,
    LR = 6,
    RL = 7,
    SL = 8,
    SR = 9,
    S2L = 10,
    S2R = 11,
    UL = 12,
    UR = 13,
    KinkL = 14,
    KinkR = 15,
}

impl TryFrom<u8> for MarkerCornerKind {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::CurveL),
            1 => Ok(Self::CurveR),
            2 => Ok(Self::L),
            3 => Ok(Self::R),
            4 => Ok(Self::HardL),
            5 => Ok(Self::HardR),
            6 => Ok(Self::LR),
            7 => Ok(Self::RL),
            8 => Ok(Self::SL),
            9 => Ok(Self::SR),
            10 => Ok(Self::S2L),
            11 => Ok(Self::S2R),
            12 => Ok(Self::UL),
            13 => Ok(Self::UR),
            14 => Ok(Self::KinkL),
            15 => Ok(Self::KinkR),
            found => Err(DecodeErrorKind::NoVariantMatch {
                found: found as u64,
            }
            .into()),
        }
    }
}

/// Corner Marker
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MarkerCorner {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Kind of marker
    pub kind: MarkerCornerKind,
    /// Heading / Direction
    pub heading: Heading,
    /// Floating
    pub floating: bool,
}

impl MarkerCorner {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = self.kind as u8 & 0x0f;
        if self.floating {
            flags |= 0x80;
        }
        ObjectFlags(flags)
    }

    pub(super) fn new(
        xyz: ObjectCoordinate,
        wire: ObjectFlags,
        heading: Heading,
    ) -> Result<Self, crate::DecodeError> {
        let kind = MarkerCornerKind::try_from(wire.0 & 0x0f)?;
        let floating = wire.floating();
        Ok(Self {
            xyz,
            kind,
            heading,
            floating,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Marker Distance Kind
pub enum MarkerDistanceKind {
    #[default]
    D25 = 0,
    D50 = 1,
    D75 = 2,
    D100 = 3,
    D125 = 4,
    D150 = 5,
    D200 = 6,
    D250 = 7,
}

impl TryFrom<u8> for MarkerDistanceKind {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::D25),
            1 => Ok(Self::D50),
            2 => Ok(Self::D75),
            3 => Ok(Self::D100),
            4 => Ok(Self::D125),
            5 => Ok(Self::D150),
            6 => Ok(Self::D200),
            7 => Ok(Self::D250),
            found => Err(DecodeErrorKind::NoVariantMatch {
                found: found as u64,
            }
            .into()),
        }
    }
}

/// Distance Marker
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MarkerDistance {
    /// Position
    pub xyz: ObjectCoordinate,
    /// Kind of distance marker
    pub kind: MarkerDistanceKind,
    /// Heading / Direction
    pub heading: Heading,
    /// Floating
    pub floating: bool,
}

impl MarkerDistance {
    pub(super) fn to_flags(&self) -> ObjectFlags {
        let mut flags = self.kind as u8 & 0x0f;
        if self.floating {
            flags |= 0x80;
        }
        ObjectFlags(flags)
    }

    pub(super) fn new(
        xyz: ObjectCoordinate,
        wire: ObjectFlags,
        heading: Heading,
    ) -> Result<Self, crate::DecodeError> {
        let kind = MarkerDistanceKind::try_from(wire.0 & 0x0f)?;
        let floating = wire.floating();
        Ok(Self {
            xyz,
            kind,
            heading,
            floating,
        })
    }
}
