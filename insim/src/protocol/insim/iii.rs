use crate::protocol::identifiers::{ConnectionId, PlayerId, RequestId};
use crate::string::CodepageString;
use deku::ctx::Size;
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
/// InsIm Info -  a /i message from user to hosts Insim
pub struct Iii {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    #[deku(pad_bytes_after = "2")]
    pub plid: PlayerId,

    #[deku(reader = "CodepageString::read(deku::rest, Size::Bytes(deku::rest.len() / 8))")]
    pub msg: CodepageString,
}
