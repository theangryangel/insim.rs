use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{
    protocol::identifiers::{ConnectionId, RequestId},
    string::{istring, CodepageString},
};

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// New Connection
pub struct Ncn {
    pub reqi: RequestId,

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
