use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use insim_core::{binrw::{self as binrw, binrw}, FromToBytes};
#[cfg(feature = "serde")]
use serde::Serialize;

/// Button Click Identifier
#[binrw]
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ClickId(pub u8);

impl fmt::Display for ClickId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for ClickId {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ClickId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u8> for ClickId {
    fn from(value: u8) -> Self {
        Self(value)
    }
}


impl FromToBytes for ClickId {
    fn from_bytes(buf: &mut Bytes) -> Result<Self, insim_core::Error> {
        Ok(ClickId(buf.get_u8()))
    }

    fn to_bytes(&self, buf: &mut BytesMut) -> Result<(), insim_core::Error> {
        buf.put_u8(self.0);

        Ok(())
    }
}
