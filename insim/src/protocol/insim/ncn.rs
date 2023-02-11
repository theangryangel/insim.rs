use insim_core::{
    identifiers::{ConnectionId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// New Connection
pub struct Ncn {
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    /// Username.
    #[insim(bytes = "24")]
    pub uname: String,

    #[insim(bytes = "24")]
    /// Playername.
    pub pname: String,

    /// 1 if administrative user.
    pub admin: bool,

    /// Total number of connections now this player has joined.
    pub total: u8,

    #[insim(pad_bytes_after = "1")]
    pub flags: u8,
}
