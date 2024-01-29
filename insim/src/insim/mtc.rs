use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string_until_eof, binrw_write_codepage_string},
};

use super::SoundType;
use crate::identifiers::{ConnectionId, PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Message to Connection - Send a message to a specific connection, restricted to hosts only
pub struct Mtc {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// See [SoundType].
    pub sound: SoundType,

    /// Unique connection id
    pub ucid: ConnectionId,

    /// Unique player id
    #[brw(pad_after = 2)]
    pub plid: PlayerId,

    /// Message
    #[bw(write_with = binrw_write_codepage_string::<128, _>, args(false, 4))]
    #[br(parse_with = binrw_parse_codepage_string_until_eof)]
    pub text: String,
}

#[cfg(test)]
mod tests {
    use insim_core::binrw::BinWrite;
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_mtc_valid() {
        let data = Mtc {
            reqi: RequestId(1),
            plid: PlayerId(0),
            ucid: ConnectionId(0),
            sound: SoundType::default(),
            text: "aaaaa".into(),
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
