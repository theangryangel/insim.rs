use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, PlayerId, RequestId},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
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

    // FIXME - should be dynamically sized
    #[br(parse_with = binrw_parse_codepage_string::<64,_>)]
    #[bw(write_with = binrw_write_codepage_string::<64, _>)]
    pub msg: String,
}
