//! Identifiers

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

pub use ::insim_core as core;
use bytes::{Buf, BufMut};
use insim_core::{Decode, Encode};

/// Unique Player Identifier, commonly referred to as PLID in Insim.txt
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OutsimId(pub i32);

impl fmt::Display for OutsimId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for OutsimId {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OutsimId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<i32> for OutsimId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl Decode for OutsimId {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        Ok(OutsimId(buf.get_i32_le()))
    }
}

impl Encode for OutsimId {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        buf.put_i32_le(self.0);

        Ok(())
    }
}
