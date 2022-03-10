use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Split timing
pub struct Spx {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub plid: u8,

    #[deku(bytes = "4")]
    pub stime: u32,

    #[deku(bytes = "4")]
    pub etime: u32,

    #[deku(bytes = "1")]
    pub split: u8,

    #[deku(bytes = "1")]
    pub penalty: u8,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub numstops: u8,
}
