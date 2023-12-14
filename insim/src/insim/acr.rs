use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, RequestId},
    string::{binrw_parse_codepage_string_until_eof, binrw_write_codepage_string},
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
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub admin: bool,

    #[brw(pad_after = 1)]
    pub result: AcrResult,

    #[bw(write_with = binrw_write_codepage_string::<64, _>, args(false, 4))]
    #[br(parse_with = binrw_parse_codepage_string_until_eof)]
    pub text: String,
}
