use insim_core::{
    binrw::{self, binrw},
    string::{binrw_parse_codepage_string_until_eof, binrw_write_codepage_string},
};

use crate::identifiers::{ConnectionId, RequestId};

/// Enum for the result field of [Acr].
#[binrw]
#[brw(repr(u8))]
#[repr(u8)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub enum AcrResult {
    /// Command was processed
    #[default]
    Processed = 1,

    /// Command was rejected
    Rejected = 2,

    /// Unknown command
    UnknownCommand = 3,
}

/// Admin Command Report: A user typed an admin command - variable size
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Acr {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// Unique connection identifier
    pub ucid: ConnectionId,

    /// Is the user an admin?
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub admin: bool,

    /// Result
    #[brw(pad_after = 1)]
    pub result: AcrResult,

    /// Command
    #[bw(write_with = binrw_write_codepage_string::<64, _>, args(false, 4))]
    #[br(parse_with = binrw_parse_codepage_string_until_eof)]
    pub text: String,
}
