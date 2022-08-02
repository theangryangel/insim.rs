use crate::{protocol::identifiers::ConnectionId, string::CodepageString};
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
/// Connection Player Renamed indicates that a player has changed their name.
pub struct Cpr {
    pub reqi: u8,

    pub ucid: ConnectionId,

    #[deku(bytes = "24")]
    pub pname: CodepageString,

    #[deku(bytes = "8")]
    pub plate: CodepageString,
}
