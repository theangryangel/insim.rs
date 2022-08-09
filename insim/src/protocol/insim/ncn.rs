use crate::{
    protocol::identifiers::ConnectionId,
    string::{istring, CodepageString},
};
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
/// New Connection
pub struct Ncn {
    pub reqi: u8,

    pub ucid: ConnectionId,

    /// Username.
    #[deku(
        reader = "istring::read(deku::rest, 24)",
        writer = "istring::write(deku::output, &self.uname, 24)"
    )]
    pub uname: String,

    #[deku(bytes = "24")]
    /// Playername.
    pub pname: CodepageString,

    /// 1 if administrative user.
    pub admin: u8,

    /// Total number of connections now this player has joined.
    pub total: u8,

    #[deku(pad_bytes_after = "1")]
    pub flags: u8,
}
