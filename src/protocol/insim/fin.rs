use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Fin {
    #[deku(bytes = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    plid: u8,

    #[deku(bytes = "4")]
    ttime: u32,

    #[deku(bytes = "4", pad_bytes_after = "1")]
    btime: u32,

    #[deku(bytes = "1")]
    numstops: u8,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    confirm: u8,

    #[deku(bytes = "2")]
    lapsdone: u16,

    #[deku(bytes = "2")]
    flags: u16,
    // unsigned * 2
}
