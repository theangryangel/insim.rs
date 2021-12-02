use super::PlayerFlags;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Lap Time
pub struct Lap {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub plid: u8,

    #[deku(bytes = "4")]
    pub ltime: u32, // lap time (ms)

    #[deku(bytes = "4")]
    pub etime: u32,

    #[deku(bytes = "2")]
    pub lapsdone: u16,

    #[deku(bytes = "2", pad_bytes_after = "1")]
    pub flags: PlayerFlags,

    #[deku(bytes = "1")]
    pub penalty: u8,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub numstops: u8,
}
