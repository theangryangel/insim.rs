use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, PlayerId, RequestId},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the sound field of [Mso].
#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum MsoUserType {
    /// System message.
    #[default]
    System = 0,

    /// Normal, visible, user message.
    User = 1,

    /// Was this message received with the prefix character from the [Isi](super::Isi) message?
    Prefix = 2,

    // Hidden message (due to be retired in Insim v9)
    O = 3,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// System messsages and user messages, variable sized.
pub struct Mso {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    pub ucid: ConnectionId,
    pub plid: PlayerId,
    /// Set if typed by a user
    pub usertype: MsoUserType,
    /// Index of the first character of user entered text, in msg field.
    pub textstart: u8,

    #[bw(write_with = binrw_write_codepage_string::<128, _>, args(false, 4))]
    #[br(parse_with = binrw_parse_codepage_string::<128, _>)]
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use super::{Mso, MsoUserType};
    use crate::core::identifiers::{ConnectionId, PlayerId, RequestId};
    use bytes::{BufMut, BytesMut};
    use insim_core::binrw::BinWrite;
    use std::io::Cursor;

    #[test]
    fn test_mso() {
        let data = Mso {
            reqi: RequestId(1),
            ucid: ConnectionId(10),
            plid: PlayerId(74),
            usertype: MsoUserType::System,
            textstart: 0,
            msg: "two".into(),
        };

        let mut buf = Cursor::new(Vec::new());
        let res = data.write_le(&mut buf);
        assert!(res.is_ok());

        let mut comparison = BytesMut::new();
        comparison.put_u8(1);
        comparison.put_u8(0);
        comparison.put_u8(10);
        comparison.put_u8(74);
        comparison.put_u8(0);
        comparison.put_u8(0);
        comparison.extend_from_slice(&"two".to_string().as_bytes());
        comparison.put_bytes(0, 1);

        assert_eq!(buf.into_inner(), comparison.to_vec());
    }
}
