use insim_core::{identifiers::RequestId, binrw::{self, binrw}, string::{binrw_write_codepage_string, binrw_parse_codepage_string}};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Message Type - Send a message to LFS as if typed by a user
pub struct Mst {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    // FIXME: ensure its nul terminated
    #[bw(write_with = binrw_write_codepage_string::<64, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<64, _>)]
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::Mst;
    use crate::core::identifiers::RequestId;

    #[test]
    fn ensure_last_byte_zero_always() {
        let data = Mst {
            reqi: RequestId(1),
            msg: "aaaaaa".into(),
        };

        let mut buf = BytesMut::new();
        let res = data.encode(&mut buf, None);
        assert!(res.is_ok());

        assert_eq!(buf.last(), Some(&0));
        assert_eq!(buf.len(), 66);
    }
}
