use insim_core::{
    identifiers::{ConnectionId, RequestId},
    prelude::*,
};

/// Enum for the flag field of [Flg].
#[derive(Default, Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum VtnAction {
    #[default]
    None = 0,

    End = 1,

    Restart = 2,

    Qualify = 3,
}

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
    pub action: VtnAction,
}
