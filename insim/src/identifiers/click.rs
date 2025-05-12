use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use insim_core::{Decode, Encode};
#[cfg(feature = "serde")]
use serde::Serialize;

/// Button Click Identifier
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ClickId(pub u8);

impl ClickId {
    const MAX: u8 = 239;
}

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

impl Decode for ClickId {
    fn decode(buf: &mut Bytes) -> Result<Self, insim_core::DecodeError> {
        let clickid = buf.get_u8();
        if clickid > Self::MAX {
            Err(insim_core::DecodeError::TooLarge)
        } else {
            Ok(ClickId(clickid))
        }
    }
}

impl Encode for ClickId {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), insim_core::EncodeError> {
        if self.0 > Self::MAX {
            Err(insim_core::EncodeError::TooLarge)
        } else {
            buf.put_u8(self.0);
            Ok(())
        }
    }
}
