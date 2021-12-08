use super::PlayerFlags;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Provisional finish notification: This is not a final result, you should use the [Res](super::Res) packet for this instead.
pub struct Fin {
    pub reqi: u8,

    pub plid: u8,

    pub ttime: u32,

    #[deku(pad_bytes_after = "1")]
    pub btime: u32,

    pub numstops: u8,

    #[deku(pad_bytes_after = "1")]
    pub confirm: u8,

    pub lapsdone: u16,

    pub flags: PlayerFlags,
}
