use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use bytes::{Buf, BufMut};
use insim_core::{Decode, DecodeContext, Encode, EncodeContext};

/// Button Click Identifier
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClickId(pub u8);

impl ClickId {
    /// Maximum supported value for ClickId
    pub const MAX: u8 = 239;
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
    const PRIMITIVE: bool = true;
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let clickid = ctx.buf.get_u8();
        if clickid > Self::MAX {
            Err(insim_core::DecodeErrorKind::OutOfRange {
                min: 1,
                max: Self::MAX as usize,
                found: clickid as usize,
            }
            .into())
        } else {
            Ok(ClickId(clickid))
        }
    }
}

impl Encode for ClickId {
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        if self.0 > Self::MAX {
            Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 1,
                max: Self::MAX as usize,
                found: self.0 as usize,
            }
            .into())
        } else {
            ctx.buf.put_u8(self.0);
            Ok(())
        }
    }
}
