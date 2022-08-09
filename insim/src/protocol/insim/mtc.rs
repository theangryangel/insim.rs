use crate::protocol::identifiers::{ConnectionId, PlayerId};
use crate::string::CodepageString;
use deku::ctx::Size;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Message to Connection - Send a message to a specific connection, restricted to hosts only
pub struct Mtc {
    pub reqi: u8,

    pub sound: u8,

    pub ucid: ConnectionId,

    #[deku(pad_bytes_after = "2")]
    pub plid: PlayerId,

    #[deku(reader = "CodepageString::read(deku::rest, Size::Bytes(deku::rest.len() / 8))")]
    pub msg: CodepageString,
}
