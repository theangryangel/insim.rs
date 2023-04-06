use insim_core::{
    identifiers::{ConnectionId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Default, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
/// Used within [Cnl] to indicate the leave reason.
pub enum CnlReason {
    #[default]
    Disconnected = 0,

    Timeout = 1,

    LostConnection = 2,

    Kicked = 3,

    Banned = 4,

    Security = 5,

    Cpw = 6,

    Oos = 7,

    Joos = 8,

    Hack = 9,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
// Connection Leave
pub struct Cnl {
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    pub reason: CnlReason,

    #[insim(pad_bytes_after = "2")]
    pub total: u8,
}
