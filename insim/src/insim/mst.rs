use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
    ReadWriteBuf, FromToCodepageBytes,
};

use crate::identifiers::RequestId;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Send a message to LFS as if typed by a user
pub struct Mst {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// Message
    #[bw(write_with = binrw_write_codepage_string::<64, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<64, _>)]
    pub msg: String,
}

impl ReadWriteBuf for Mst {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        buf.advance(1);
        let msg = String::from_codepage_bytes(buf, 64)?;
        Ok(Self { reqi, msg })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        buf.put_bytes(0, 1);
        self.msg.to_codepage_bytes(buf, 64)?;
        Ok(())
    }
}

impl_typical_with_request_id!(Mst);

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_mst() {
        let mut raw = BytesMut::new();
        raw.extend_from_slice(&[1, 0, b'a', b'b', b'c', b'd', b'e', b'f']);
        raw.put_bytes(0, 64 + 2 - raw.len());

        assert_from_to_bytes!(Mst, raw.as_ref(), |parsed: Mst| {
            assert_eq!(parsed.msg, "abcdef".to_string());
        });
    }
}
