use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use insim_core::{
    binrw::{self as binrw, binrw},
    ReadWriteBuf,
};
#[cfg(feature = "serde")]
use serde::Serialize;

/// Request Identifier, commonly referred to as reqi in Insim.txt
#[binrw]
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RequestId(pub u8);

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for RequestId {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RequestId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u8> for RequestId {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl ReadWriteBuf for RequestId {
    fn read_buf(buf: &mut Bytes) -> Result<Self, insim_core::Error> {
        Ok(RequestId(buf.get_u8()))
    }

    fn write_buf(&self, buf: &mut BytesMut) -> Result<(), insim_core::Error> {
        buf.put_u8(self.0);

        Ok(())
    }
}
