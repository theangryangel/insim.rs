use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{PlayerId, RequestId};

use super::PlayerFlags;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Player Flags
pub struct Pfl {
    pub reqi: RequestId,

    pub plid: PlayerId,

    #[deku(pad_bytes_after = "2")]
    pub flags: PlayerFlags,
}
