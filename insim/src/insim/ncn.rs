use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, RequestId},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// New Connection
pub struct Ncn {
    pub reqi: RequestId,
    pub ucid: ConnectionId,

    /// Username.
    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    pub uname: String,

    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    /// Playername.
    pub pname: String,

    /// true if administrative user.
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub admin: bool,

    /// Total number of connections now this player has joined, plus host
    pub total: u8,

    #[brw(pad_after = 1)]
    pub flags: u8,
}
