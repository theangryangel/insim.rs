use insim_core::{
    identifiers::{ConnectionId, RequestId},
    prelude::*,
    string::CodepageString,
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the result field of [Acr].
#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum AcrResult {
    None = 0,

    Processed = 1,

    Rejected = 2,

    UnknownCommand = 3,
}

impl Default for AcrResult {
    fn default() -> Self {
        AcrResult::None
    }
}

/// Admin Command Report
#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Acr {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    pub admin: u8,

    #[insim(pad_bytes_after = "1")]
    pub result: AcrResult,

    #[insim(bytes = "64")]
    pub text: CodepageString,
}
