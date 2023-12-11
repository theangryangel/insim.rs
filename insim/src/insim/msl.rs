use insim_core::{
    binrw::{self, binrw},
    identifiers::RequestId,
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the sound field of [Msl].
#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum SoundType {
    #[default]
    Silent = 0,

    Message = 1,

    SystemMessage = 2,

    InvalidKey = 3,

    // This is referred to as "Error" in the Insim documentation, but this is a special word in
    // rust so I'm trying to avoid it.
    Failure = 4,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Send a message to the local computer only. If you are connected to a server this means the
/// console. If you are connected to a client this means to the local client only.
pub struct Msl {
    pub reqi: RequestId,

    pub sound: SoundType,

    #[bw(write_with = binrw_write_codepage_string::<128, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<128, _>)]
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use insim_core::binrw::BinWrite;

    use super::{Msl, SoundType};
    use crate::core::identifiers::RequestId;

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
