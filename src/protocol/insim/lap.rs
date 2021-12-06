use super::PlayerFlags;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Lap Time for a given player.
pub struct Lap {
    pub reqi: u8,

    pub plid: u8,

    pub ltime: u32, // lap time (ms)

    pub etime: u32,

    pub lapsdone: u16,

    #[deku(pad_bytes_after = "1")]
    pub flags: PlayerFlags,

    pub penalty: u8,

    #[deku(pad_bytes_after = "1")]
    pub numstops: u8,
}
