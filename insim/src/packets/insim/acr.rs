use insim_core::{
    identifiers::{ConnectionId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the result field of [Acr].
#[derive(Debug, Default, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum AcrResult {
    #[default]
    None = 0,

    Processed = 1,

    Rejected = 2,

    UnknownCommand = 3,
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
    pub text: String,
}
