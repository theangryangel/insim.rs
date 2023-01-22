use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{ConnectionId, PlayerId, RequestId};
use crate::string::CodepageString;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Message to Connection - Send a message to a specific connection, restricted to hosts only
pub struct Mtc {
    pub reqi: RequestId,

    pub sound: u8,

    pub ucid: ConnectionId,

    #[deku(pad_bytes_after = "2")]
    pub plid: PlayerId,

    #[deku(reader = "CodepageString::read(deku::rest, Size::Bytes(deku::rest.len() / 8))")]
    pub msg: CodepageString,
}
