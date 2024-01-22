use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

use crate::identifiers::RequestId;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Extended Message (like [Mst](super::Mst), but longer)
pub struct Msx {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// Message
    #[bw(write_with = binrw_write_codepage_string::<96, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<96, _>)]
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use insim_core::binrw::BinWrite;
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_msx() {
        let data = Msx {
            reqi: RequestId(1),
            msg: "aaaaaa".into(),
        };

        let mut buf = Cursor::new(Vec::new());
        let res = data.write_le(&mut buf);
        assert!(res.is_ok());
        let buf = buf.into_inner();

        assert_eq!(buf.last(), Some(&0));
        assert_eq!(buf.len(), 98);
    }
}
