use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{PlayerId, RequestId};

use super::PlayerFlags;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Lap Time for a given player.
pub struct Lap {
    pub reqi: RequestId,

    pub plid: PlayerId,

    pub ltime: u32, // lap time (ms)

    pub etime: u32,

    pub lapsdone: u16,

    #[insim(pad_bytes_after = "1")]
    pub flags: PlayerFlags,

    pub penalty: u8,

    #[insim(pad_bytes_after = "1")]
    pub numstops: u8,
}
