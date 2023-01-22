use crate::protocol::identifiers::{ConnectionId, RequestId};
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
/// Used within [Cnl] to indicate the leave reason.
pub enum CnlReason {
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

impl Default for CnlReason {
    fn default() -> Self {
        CnlReason::Disconnected
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
// Connection Leave
pub struct Cnl {
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    pub reason: CnlReason,

    #[deku(pad_bytes_after = "2")]
    pub total: u8,
}
