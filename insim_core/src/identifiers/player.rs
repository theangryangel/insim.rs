use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};

/// Unique Player Identifier, commonly referred to as PLID in Insim.txt
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl crate::Decode for PlayerId {
    fn decode(buf: &mut Bytes) -> Result<Self, crate::DecodeError> {
        Ok(PlayerId(buf.get_u8()))
    }
}

impl crate::Encode for PlayerId {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), crate::EncodeError> {
        buf.put_u8(self.0);

        Ok(())
    }
}
