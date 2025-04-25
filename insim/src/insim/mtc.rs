use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string_until_eof, binrw_write_codepage_string},
};

use super::SoundType;
use crate::identifiers::{ConnectionId, PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
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
    #[read_write_buf(pad_after = 2)]
    pub plid: PlayerId,

    /// Message
    #[bw(write_with = binrw_write_codepage_string::<128, _>, args(false, 4))]
    #[br(parse_with = binrw_parse_codepage_string_until_eof)]
    #[read_write_buf(codepage(length = 128, align_to = 4))]
    pub text: String,
}

impl_typical_with_request_id!(Mtc);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mtc() {
        let raw = [
            1, // reqi
            1, // soundtype
            0, // ucid
            2, // plid
            0, 0, b'a', b'b', b'c', b'd', b'e', 0, 0, 0,
        ];

        assert_from_to_bytes!(Mtc, raw, |parsed: Mtc| {
            assert_eq!(parsed.reqi, RequestId(1));
            assert_eq!(parsed.plid, PlayerId(2));
            assert_eq!(parsed.ucid, ConnectionId(0));
            assert_eq!(parsed.sound, SoundType::Message);
            assert_eq!(&parsed.text, "abcde");
        });
    }
}
