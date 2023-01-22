use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{ConnectionId, RequestId};

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Vote Notification
pub struct Vtn {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    #[deku(pad_bytes_after = "2")]
    pub action: u8,
}
