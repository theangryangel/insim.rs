use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw},
    FromToBytes,
};

use crate::identifiers::RequestId;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Screen Mode (referred to as originally IS_MOD within Insim.txt)
pub struct Mod {
    #[brw(pad_after = 1)]
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Set to choose 16-bit
    pub bit16: i32,

    /// Refresh rate, zero for default
    pub rr: i32,

    /// Screen width. Zero to switch to windowed mode.
    pub width: i32,

    /// Screen height. Zero to switch to windowed mode.
    pub height: i32,
}

impl FromToBytes for Mod {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::from_bytes(buf)?;
        buf.advance(1);
        let bit16 = i32::from_bytes(buf)?;
        let rr = i32::from_bytes(buf)?;
        let width = i32::from_bytes(buf)?;
        let height = i32::from_bytes(buf)?;
        Ok(Self {
            reqi,
            bit16,
            rr,
            width,
            height,
        })
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.to_bytes(buf)?;
        buf.put_u8(0);
        self.bit16.to_bytes(buf)?;
        self.rr.to_bytes(buf)?;
        self.width.to_bytes(buf)?;
        self.height.to_bytes(buf)?;

        Ok(())
    }
}

impl_typical_with_request_id!(Mod);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mod() {
        let raw = [
            0,   // reqi
            0,   // zero
            2,   // bits16 (1)
            0,   // bits16 (2)
            0,   // bits16 (3)
            0,   // bits16 (4)
            59,  // rr (1)
            0,   // rr (2)
            0,   // rr (3)
            0,   // rr (4)
            128, // width (1)
            7,   // width (2)
            0,   // width (3)
            0,   // width (4)
            56,  // height (1)
            4,   // height (2)
            0,   // height (3)
            0,   // height (4)
        ];
        assert_from_to_bytes!(Mod, raw, |parsed: Mod| {
            assert_eq!(parsed.reqi, RequestId(0));
            assert_eq!(parsed.bit16, 2);
            assert_eq!(parsed.rr, 59);
            assert_eq!(parsed.width, 1920);
            assert_eq!(parsed.height, 1080);
        });
    }
}
