use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{ConnectionId, PlayerId, RequestId};

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Take Over Car
pub struct Toc {
    pub reqi: RequestId,

    pub plid: PlayerId,

    pub olducid: ConnectionId,

    #[insim(pad_bytes_after = "2")]
    pub newucid: ConnectionId,
}
