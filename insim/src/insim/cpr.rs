use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

use crate::identifiers::{ConnectionId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Connection Player Renamed indicates that a player has changed their name or number plate.
pub struct Cpr {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection ID of the connection that was renamed
    pub ucid: ConnectionId,

    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    /// New player name
    pub pname: String,

    #[br(parse_with = binrw_parse_codepage_string::<8, _>)]
    #[bw(write_with = binrw_write_codepage_string::<8, _>)]
    /// New number plate
    pub plate: String,
}
