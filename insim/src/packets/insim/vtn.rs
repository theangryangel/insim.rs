use insim_core::{
    identifiers::{ConnectionId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Vote Notification
pub struct Vtn {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    #[insim(pad_bytes_after = "2")]
    pub action: u8,
}
