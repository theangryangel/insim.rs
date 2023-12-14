use insim_core::{
    binrw::{self, binrw},
    identifiers::RequestId,
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Message Type - Send a message to LFS as if typed by a user
pub struct Mst {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    #[bw(write_with = binrw_write_codepage_string::<64, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<64, _>)]
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use super::Mst;
    use crate::core::identifiers::RequestId;
    use insim_core::binrw::BinWrite;
    use std::io::Cursor;

    #[test]
    fn test_mst() {
        let data = Mst {
            reqi: RequestId(1),
            msg: "aaaaaa".into(),
        };

        let mut buf = Cursor::new(Vec::new());
        let res = data.write_le(&mut buf);
        assert!(res.is_ok());
        let buf = buf.into_inner();

        assert_eq!(buf.last(), Some(&0));
        assert_eq!(buf.len(), 66);
    }
}
