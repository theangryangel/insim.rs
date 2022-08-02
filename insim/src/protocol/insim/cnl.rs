use crate::protocol::identifiers::ConnectionId;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    type = "u8",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Used within [Cnl] to indicate the leave reason.
pub enum CnlReason {
    #[deku(id = "0")]
    Disconnected,

    #[deku(id = "1")]
    Timeout,

    #[deku(id = "2")]
    LostConnection,

    #[deku(id = "3")]
    Kicked,

    #[deku(id = "4")]
    Banned,

    #[deku(id = "5")]
    Security,

    #[deku(id = "6")]
    Cpw,

    #[deku(id = "7")]
    Oos,

    #[deku(id = "8")]
    Joos,

    #[deku(id = "9")]
    Hack,
}

impl Default for CnlReason {
    fn default() -> Self {
        CnlReason::Disconnected
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
// Connection Leave
pub struct Cnl {
    pub reqi: u8,

    pub ucid: ConnectionId,

    pub reason: CnlReason,

    #[deku(pad_bytes_after = "2")]
    pub total: u8,
}
