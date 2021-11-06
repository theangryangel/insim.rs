use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct SplitX {
    #[deku(bytes = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    plid: u8,

    #[deku(bytes = "4")]
    stime: u32,

    #[deku(bytes = "4")]
    etime: u32,

    #[deku(bytes = "1")]
    split: u8,

    #[deku(bytes = "1")]
    penalty: u8,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    numstops: u8,
}
