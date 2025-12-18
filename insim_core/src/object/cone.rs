//! Control objects
use super::ObjectVariant;
use crate::{DecodeError, direction::Direction};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
#[non_exhaustive]
/// Cone Kind
pub enum ConeKind {
    #[default]
    One = 20,
    Two = 21,
    TallOne = 32,
    TallTwo = 33,
    Pointer = 40,
}

impl TryFrom<u8> for ConeKind {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            20 => Ok(Self::One),
            21 => Ok(Self::Two),
            32 => Ok(Self::TallOne),
            33 => Ok(Self::TallTwo),
            40 => Ok(Self::Pointer),
            found => Err(DecodeError::NoVariantMatch {
                found: found as u64,
            }),
        }
    }
}

/// Cones stack
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Cone {
    /// Kind of cone
    pub kind: ConeKind,
    /// Heading / Direction
    pub heading: Direction,
    /// Floating
    pub floating: bool,
}

impl ObjectVariant for Cone {
    fn encode(&self) -> Result<(u8, u8, u8), crate::EncodeError> {
        todo!()
    }

    fn decode(_index: u8, _flags: u8, _heading: u8) -> Result<Self, crate::DecodeError> {
        todo!()
    }
}
