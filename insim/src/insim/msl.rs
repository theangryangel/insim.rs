use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

use crate::identifiers::RequestId;

/// Enum for the sound field of [Msl].
#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
#[non_exhaustive]
pub enum SoundType {
    #[default]
    /// Silent
    Silent = 0,

    /// Message "ping"
    Message = 1,

    /// System message "ping"
    SysMessage = 2,

    /// Invalid key "ping"
    InvalidKey = 3,

    /// Error "ping"
    Error = 4,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Send a message to the local computer only. If you are connected to a server this means the
/// console. If you are connected to a client this means to the local client only.
pub struct Msl {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// See [SoundType]
    pub sound: SoundType,

    /// Message
    #[bw(write_with = binrw_write_codepage_string::<128, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<128, _>)]
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use insim_core::binrw::BinWrite;

    use super::{Msl, SoundType};
    use crate::identifiers::RequestId;

    #[test]
    fn test_msl() {
        let data = Msl {
            reqi: RequestId(1),
            sound: SoundType::Silent,
            msg: "aaaaaa".into(),
        };

        let mut buf = Cursor::new(Vec::new());
        let res = data.write_le(&mut buf);
        assert!(res.is_ok());
        let buf = buf.into_inner();

        assert_eq!(buf.last(), Some(&0));
        assert_eq!(buf.len(), 130);
    }
}
