use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::ConnectionId;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Vote Notification
pub struct Vtn {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    pub ucid: ConnectionId,

    #[deku(pad_bytes_after = "2")]
    pub action: u8,
}
