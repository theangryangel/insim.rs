use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, PlayerId, RequestId},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

pub use super::SoundType;

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Message to Connection - Send a message to a specific connection, restricted to hosts only
pub struct Mtc {
    pub reqi: RequestId,
    pub sound: SoundType,

    pub ucid: ConnectionId,
    #[brw(pad_after = 2)]
    pub plid: PlayerId,

    #[bw(write_with = binrw_write_codepage_string::<128, _>, args(false, 4))]
    #[br(parse_with = binrw_parse_codepage_string::<128, _>, args(false))]
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use insim_core::{
        binrw::BinWrite,
        identifiers::{ConnectionId, PlayerId},
    };
    use std::io::Cursor;

    use super::{Mtc, SoundType};
    use crate::core::identifiers::RequestId;

    #[test]
    fn test_mtc_valid() {
        let data = Mtc {
            reqi: RequestId(1),
            plid: PlayerId(0),
            ucid: ConnectionId(0),
            sound: SoundType::default(),
            msg: "aaaaa".into(),
        };

        let mut buf = Cursor::new(Vec::new());
        let res = data.write_le(&mut buf);
        assert!(res.is_ok());
        let buf = buf.into_inner();

        assert_eq!((buf.len() - 6) % 4, 0);

        assert_eq!(buf.last(), Some(&0));
        assert_eq!(buf.len(), 14);
    }
}
