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

    #[insim(pad_bytes_after = "2")]
    pub plid: PlayerId,

    #[insim(reader = "CodepageString::read(insim::rest, Size::Bytes(insim::rest.len() / 8))")]
    pub msg: CodepageString,
}
