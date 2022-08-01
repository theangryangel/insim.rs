use crate::protocol::identifiers::ConnectionId;
use crate::string::CodepageString;
use deku::ctx::Size;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// InsIm Info -  a /i message from user to hosts Insim
pub struct Iii {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    pub ucid: ConnectionId,

    #[deku(pad_bytes_after = "2")]
    pub plid: u8,

    #[deku(reader = "CodepageString::read(deku::rest, Size::Bytes(deku::rest.len() / 8))")]
    pub msg: CodepageString,
}
