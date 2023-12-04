use insim_core::{
    identifiers::{ConnectionId, RequestId},
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the result field of [Acr].
#[binrw]
#[brw(repr(u8))]
#[repr(u8)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum AcrResult {
    #[default]
    None = 0,

    Processed = 1,

    Rejected = 2,

    UnknownCommand = 3,
}

/// Admin Command Report
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Acr {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    pub ucid: ConnectionId,
    pub admin: u8, // FIXME should be bool

    #[brw(pad_after = 1)]
    pub result: AcrResult,

    #[br(parse_with = binrw_parse_codepage_string::<64, _>)]
    #[bw(write_with = binrw_write_codepage_string::<64, _>)]
    pub text: String,
}
