use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use insim_core::{Decode, Encode};
#[cfg(feature = "serde")]
use serde::Serialize;

/// Unique Player Identifier, commonly referred to as PLID in Insim.txt
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct PlayerId(pub u8);

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for PlayerId {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PlayerId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u8> for PlayerId {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl Decode for PlayerId {
    fn decode(buf: &mut Bytes) -> Result<Self, insim_core::DecodeError> {
        Ok(PlayerId(buf.get_u8()))
    }
}

impl Encode for PlayerId {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), insim_core::EncodeError> {
        buf.put_u8(self.0);

        Ok(())
    }
}
