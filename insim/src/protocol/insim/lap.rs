use crate::protocol::identifiers::{PlayerId, RequestId};

use super::PlayerFlags;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Lap Time for a given player.
pub struct Lap {
    pub reqi: RequestId,

    pub plid: PlayerId,

    pub ltime: u32, // lap time (ms)

    pub etime: u32,

    pub lapsdone: u16,

    #[deku(pad_bytes_after = "1")]
    pub flags: PlayerFlags,

    pub penalty: u8,

    #[deku(pad_bytes_after = "1")]
    pub numstops: u8,
}
