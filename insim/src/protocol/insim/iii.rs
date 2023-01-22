use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{ConnectionId, PlayerId, RequestId};
use crate::string::CodepageString;

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// InsIm Info -  a /i message from user to hosts Insim
pub struct Iii {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    #[insim(pad_bytes_after = "2")]
    pub plid: PlayerId,

    #[insim(reader = "CodepageString::read(insim::rest, Size::Bytes(insim::rest.len() / 8))")]
    pub msg: CodepageString,
}
