//! Strongly typed wind strength
#[cfg(feature = "serde")]
use serde::Serialize;

use binrw::binrw;

use crate::FromToBytes;

#[binrw]
#[brw(repr(u8))]
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

impl FromToBytes for Wind {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, crate::Error> {
        match u8::from_bytes(buf)? {
            0 => Ok(Wind::None),
            1 => Ok(Self::Weak),
            2 => Ok(Self::Strong),
            other => Err(crate::Error::NoVariantMatch{found: other as u64})
        }
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::Error> {
        (*self as u8).to_bytes(buf)
    }
}
