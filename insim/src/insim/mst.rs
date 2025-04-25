use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

use crate::identifiers::RequestId;

#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Send a message to LFS as if typed by a user
pub struct Mst {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    #[read_write_buf(pad_after = 1)]
    pub reqi: RequestId,

    /// Message
    #[bw(write_with = binrw_write_codepage_string::<64, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<64, _>)]
    #[read_write_buf(codepage(length = 64))]
    pub msg: String,
}

impl_typical_with_request_id!(Mst);

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};

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
