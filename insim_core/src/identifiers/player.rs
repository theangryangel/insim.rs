use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use bytes::{Buf, BufMut};

use crate::{DecodeContext, EncodeContext};

/// Unique Player Identifier, commonly referred to as PLID in Insim.txt
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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
    const PRIMITIVE: bool = true;
    fn decode(ctx: &mut DecodeContext) -> Result<Self, crate::DecodeError> {
        Ok(Self(ctx.buf.get_u8()))
    }
}

impl crate::Encode for PlayerId {
    const PRIMITIVE: bool = true;
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), crate::EncodeError> {
        ctx.buf.put_u8(self.0);
        Ok(())
    }
}
