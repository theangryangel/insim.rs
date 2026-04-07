use std::{fmt, ops::Deref};

use bytes::{Buf, BufMut};
use insim_core::{Decode, DecodeContext, Encode, EncodeContext};

/// Request Identifier, commonly referred to as reqi in Insim.txt
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl From<u8> for RequestId {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl Decode for RequestId {
    const PRIMITIVE: bool = true;
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        Ok(RequestId(ctx.buf.get_u8()))
    }
}

impl Encode for RequestId {
    const PRIMITIVE: bool = true;
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.buf.put_u8(self.0);
        Ok(())
    }
}
