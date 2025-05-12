//! Strongly typed wind strength
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{Decode, Encode};

#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
/// Wind strength levels within LFS
pub enum Wind {
    #[default]
    /// No wind
    None = 0,
    /// Weak wind
    Weak = 1,
    /// Strong wind
    Strong = 2,
}

impl Decode for Wind {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::DecodeError> {
        match u8::decode(buf)? {
            0 => Ok(Wind::None),
            1 => Ok(Self::Weak),
            2 => Ok(Self::Strong),
            other => Err(crate::DecodeError::NoVariantMatch {
                found: other as u64,
            }),
        }
    }
}

impl Encode for Wind {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::EncodeError> {
        (*self as u8).encode(buf)
    }
}
