use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::ConnectionId;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Take Over Car
pub struct Toc {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub plid: u8,

    pub olducid: ConnectionId,

    #[deku(pad_bytes_after = "2")]
    pub newucid: ConnectionId,
}
