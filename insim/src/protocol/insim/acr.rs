use crate::protocol::identifiers::{ConnectionId, RequestId};
use crate::string::CodepageString;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the result field of [Acr].
#[derive(Debug, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    type = "u8",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub enum AcrResult {
    #[deku(id = "0")]
    None,

    #[deku(id = "1")]
    Processed,

    #[deku(id = "2")]
    Rejected,

    #[deku(id = "3")]
    UnknownCommand,
}

impl Default for AcrResult {
    fn default() -> Self {
        AcrResult::None
    }
}

/// Admin Command Report
#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub struct Acr {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    pub admin: u8,

    #[deku(pad_bytes_after = "1")]
    pub result: AcrResult,

    #[deku(bytes = "64")]
    pub text: CodepageString,
}
