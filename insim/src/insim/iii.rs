use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string_until_eof, binrw_write_codepage_string},
    FromToCodepageBytes,
};

use crate::identifiers::{ConnectionId, PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// InsIm Info -  a /i message from user to hosts Insim
pub struct Iii {
    #[brw(pad_after = 1)]
    #[read_write_buf(pad_after = 1)]
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection ID that the message was received from
    pub ucid: ConnectionId,

    /// Unique player iD that the message was received from
    #[brw(pad_after = 2)]
    #[read_write_buf(pad_after = 2)]
    pub plid: PlayerId,

    /// The message
    #[bw(write_with = binrw_write_codepage_string::<64, _>, args(false, 4))]
    #[br(parse_with = binrw_parse_codepage_string_until_eof)]
    #[read_write_buf(
        read_with = "|buf| { String::from_codepage_bytes(buf, 64) }",
        write_with = "|msg: &str, buf| { msg.to_codepage_bytes_aligned(buf, 64, 4) }"
    )]
    pub msg: String,
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
