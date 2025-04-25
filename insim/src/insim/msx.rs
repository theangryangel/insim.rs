use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

use crate::identifiers::RequestId;

#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Extended Message (like [Mst](super::Mst), but longer)
pub struct Msx {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    #[read_write_buf(pad_after = 1)]
    pub reqi: RequestId,

    /// Message
    #[bw(write_with = binrw_write_codepage_string::<96, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<96, _>)]
    #[read_write_buf(codepage(length = 96))]
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_msx() {
        let mut data = BytesMut::new();
        data.extend_from_slice(&[
            1, // reqi
            0,
        ]);

        data.extend_from_slice(b"aaaaaa");
        data.put_bytes(0, 96 - 6);

        assert_from_to_bytes!(Msx, data.as_ref(), |msx: Msx| {
            assert_eq!(&msx.msg, "aaaaaa");
        });
    }
}
