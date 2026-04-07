use std::{fmt, ops::Deref};

use bytes::{Buf, BufMut};
use insim_core::{Decode, DecodeContext, Encode, EncodeContext};

/// Request Identifier (ReqI).
///
/// Controls reply correlation. When you send a packet with a non-zero `reqi`, LFS echoes
/// that value back in its reply so you can match the response to your original request.
/// When `reqi` is `0`, LFS may still send a reply but with `reqi = 0`, making it
/// indistinguishable from an unsolicited packet of the same type.
///
/// Use [`crate::WithRequestId`] to attach a request id without constructing the full
/// packet struct by hand:
///
/// ```rust,ignore
/// use insim::{WithRequestId, insim::TinyType};
/// connection.send(TinyType::Ncn.with_request_id(1));
/// ```
///
/// The echo value has no special meaning beyond correlation - any non-zero `u8` works.
/// Many applications simply always use `1` when they don't need to track multiple
/// in-flight requests simultaneously.
///
/// (Referred to as `ReqI` in InSim.txt)
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
