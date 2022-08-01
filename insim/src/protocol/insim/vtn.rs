use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::ConnectionId;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Vote Notification
pub struct Vtn {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    pub ucid: ConnectionId,

    #[deku(bytes = "1", pad_bytes_after = "2")]
    pub action: u8,
}
