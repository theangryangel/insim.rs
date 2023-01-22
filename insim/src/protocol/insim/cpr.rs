use crate::{
    protocol::identifiers::{ConnectionId, RequestId},
    string::CodepageString,
};
use insim_core::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Connection Player Renamed indicates that a player has changed their name.
pub struct Cpr {
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    #[deku(bytes = "24")]
    pub pname: CodepageString,

    #[deku(bytes = "8")]
    pub plate: CodepageString,
}
