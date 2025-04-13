use bytes::{Buf, BufMut};
use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string_until_eof, binrw_write_codepage_string},
    FromToCodepageBytes, ReadWriteBuf,
};

use crate::identifiers::{ConnectionId, PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// InsIm Info -  a /i message from user to hosts Insim
pub struct Iii {
    #[brw(pad_after = 1)]
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection ID that the message was received from
    pub ucid: ConnectionId,

    /// Unique player iD that the message was received from
    #[brw(pad_after = 2)]
    pub plid: PlayerId,

    /// The message
    #[bw(write_with = binrw_write_codepage_string::<64, _>, args(false, 4))]
    #[br(parse_with = binrw_parse_codepage_string_until_eof)]
    pub msg: String,
}

impl ReadWriteBuf for Iii {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        buf.advance(1);
        let ucid = ConnectionId::read_buf(buf)?;
        let plid = PlayerId::read_buf(buf)?;
        buf.advance(2);
        let msg = String::from_codepage_bytes(buf, 64)?;
        Ok(Self {
            reqi,
            ucid,
            plid,
            msg,
        })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        buf.put_bytes(0, 1);
        self.ucid.write_buf(buf)?;
        self.plid.write_buf(buf)?;
        buf.put_bytes(0, 2);
        self.msg.to_codepage_bytes_aligned(buf, 64, 4)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_iii() {
        let mut raw = BytesMut::new();
        raw.extend_from_slice(&[
            0, // ReqI
            0, // Zero
            2, // UCID
            4, // PLID
            0, // Sp2
            0, // Sp3
        ]);
        raw.extend_from_slice(b"abcd");

        assert_from_to_bytes!(Iii, raw.freeze(), |parsed: Iii| {
            assert_eq!(parsed.reqi, RequestId(0));
            assert_eq!(parsed.ucid, ConnectionId(2));
            assert_eq!(parsed.plid, PlayerId(4));
            assert_eq!(parsed.msg, "abcd");
        });
    }
}
