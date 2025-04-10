use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
    FromToBytes, FromToCodepageBytes,
};

use crate::identifiers::RequestId;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Insim Multiplayer - LFS sends this when a host is started or joined
pub struct Ism {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// Are we a host? false = guest, true = host
    #[brw(pad_after = 3)]
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub host: bool,

    /// Name of server joined/started
    #[br(parse_with = binrw_parse_codepage_string::<32, _>)]
    #[bw(write_with = binrw_write_codepage_string::<32, _>)]
    pub hname: String,
}

impl FromToBytes for Ism {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::from_bytes(buf)?;
        buf.advance(1);
        let host = u8::from_bytes(buf)? > 0;
        buf.advance(3);
        let hname = String::from_codepage_bytes(buf, 32)?;
        Ok(Self { reqi, host, hname })
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.to_bytes(buf)?;
        buf.put_bytes(0, 1);
        (self.host as u8).to_bytes(buf)?;
        buf.put_bytes(0, 3);
        self.hname.to_codepage_bytes(buf, 32)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ism() {
        assert_from_to_bytes!(
            Ism,
            [
                1, // reqi
                0, 1, // host
                0, 0, 0, b'a', b'B', b'c', b'd', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
            |parsed: Ism| {
                assert_eq!(parsed.reqi, RequestId(1));
                assert_eq!(parsed.host, true);
                assert_eq!(&parsed.hname, "aBcd");
            }
        )
    }
}
