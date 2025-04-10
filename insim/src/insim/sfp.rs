use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw},
    FromToBytes,
};

use super::StaFlags;
use crate::identifiers::RequestId;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// State Flags Pack
pub struct Sfp {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// The state to set/change. See [StaFlags].
    pub flag: StaFlags,

    /// Turn the state on or off
    #[brw(pad_after = 1)]
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub onoff: bool,
}

impl FromToBytes for Sfp {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::from_bytes(buf)?;
        buf.advance(1);
        let flag = StaFlags::from_bytes(buf)?;
        let onoff = u8::from_bytes(buf)? > 0;
        buf.advance(1);
        Ok(Self { reqi, flag, onoff })
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.to_bytes(buf)?;
        buf.put_u8(0);
        self.flag.to_bytes(buf)?;
        (self.onoff as u8).to_bytes(buf)?;
        buf.put_u8(0);
        Ok(())
    }
}

impl_typical_with_request_id!(Sfp);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sfp() {
        assert_from_to_bytes!(
            Sfp,
            vec![
                0,   // ReqI
                0,   // Zero
                128, // Flag (1)
                4,   // Flag (2)
                1,   // OffOn
                0,   // Sp3
            ],
            |parsed: Sfp| {
                assert_eq!(parsed.reqi, RequestId(0));
                assert_eq!(parsed.onoff, true);
            }
        );
    }
}
