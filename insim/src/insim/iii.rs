use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, PlayerId, RequestId},
    string::{binrw_parse_codepage_string_until_eof, binrw_write_codepage_string},
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// InsIm Info -  a /i message from user to hosts Insim
pub struct Iii {
    pub reqi: RequestId,

    pub ucid: ConnectionId,
    #[brw(pad_after = 2)]
    pub plid: PlayerId,

    #[bw(write_with = binrw_write_codepage_string::<64, _>, args(false, 4))]
    #[br(parse_with = binrw_parse_codepage_string_until_eof)]
    pub msg: String,
}
