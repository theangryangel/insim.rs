use insim_core::{
    identifiers::{ConnectionId, PlayerId, RequestId},
    binrw::{self, binrw},
    string::{binrw_write_codepage_string, binrw_parse_codepage_string},
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

    // FIXME: should be nul terminated, grow upto 128 bytes
    // but pad so that msg is divisible by 4
    #[bw(write_with = binrw_write_codepage_string::<128, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<128, _>)]
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use insim_core::{
        identifiers::{ConnectionId, PlayerId},
    };

    use super::{Mtc, SoundType};
    use crate::core::identifiers::RequestId;

    #[test]
    fn ensure_last_byte_zero_always() {
        let data = Mtc {
            reqi: RequestId(1),
            plid: PlayerId(0),
            ucid: ConnectionId(0),
            sound: SoundType::default(),
            msg: "aaaaa".into(),
        };

        let mut buf = BytesMut::new();
        let res = data.encode(&mut buf, None);
        assert!(res.is_ok());

        assert_eq!((buf.len() - 6) % 4, 0);

        assert_eq!(buf.last(), Some(&0));
        assert_eq!(buf.len(), 14);
    }
}
