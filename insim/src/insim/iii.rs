use crate::identifiers::{ConnectionId, PlayerId, RequestId};
use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string_until_eof, binrw_write_codepage_string},
};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// InsIm Info -  a /i message from user to hosts Insim
pub struct Iii {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection ID that the message was received from
    pub ucid: ConnectionId,

    /// Unique player iD that the message was received from
    #[brw(pad_after = 2)]
    pub plid: PlayerId,

    /// The message
    #[bw(write_with = binrw_write_codepage_string::<64, _>, args(false, 4))]
    #[br(parse_with = binrw_parse_codepage_string_until_eof)]
    pub msg: String,
}
