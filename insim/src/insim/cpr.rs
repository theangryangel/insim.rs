use insim_core::{
    identifiers::{ConnectionId, RequestId},
    binrw::{binrw, self},
    string::{binrw_write_codepage_string, binrw_parse_codepage_string}
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Connection Player Renamed indicates that a player has changed their name.
pub struct Cpr {
    pub reqi: RequestId,
    pub ucid: ConnectionId,

    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    pub pname: String,

    #[br(parse_with = binrw_parse_codepage_string::<8, _>)]
    #[bw(write_with = binrw_write_codepage_string::<8, _>)]
    pub plate: String,
}
