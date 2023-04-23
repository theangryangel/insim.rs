use insim_core::{
    identifiers::{ConnectionId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Connection Player Renamed indicates that a player has changed their name.
pub struct Cpr {
    pub reqi: RequestId,
    pub ucid: ConnectionId,

    #[insim(bytes = "24")]
    pub pname: String,
    #[insim(bytes = "8")]
    pub plate: String,
}
