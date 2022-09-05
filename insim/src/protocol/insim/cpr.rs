use crate::{
    protocol::identifiers::{ConnectionId, RequestId},
    string::CodepageString,
};
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Connection Player Renamed indicates that a player has changed their name.
pub struct Cpr {
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    #[deku(bytes = "24")]
    pub pname: CodepageString,

    #[deku(bytes = "8")]
    pub plate: CodepageString,
}
