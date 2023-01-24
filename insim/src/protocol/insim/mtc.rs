use insim_core::{
    identifiers::{ConnectionId, PlayerId, RequestId},
    prelude::*,
    string::CodepageString,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Message to Connection - Send a message to a specific connection, restricted to hosts only
pub struct Mtc {
    pub reqi: RequestId,

    pub sound: u8,

    pub ucid: ConnectionId,

    #[insim(pad_bytes_after = "2")]
    pub plid: PlayerId,

    #[insim(bytes = "128")]
    pub msg: CodepageString,
}
