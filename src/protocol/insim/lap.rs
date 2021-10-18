use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Lap {
    #[deku(bytes = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    plid: u8,

    #[deku(bytes = "4")]
    ltime: u32, // lap time (ms)

    #[deku(bytes = "4")]
    etime: u32,

    #[deku(bytes = "2")]
    lapsdone: u16,

    #[deku(bytes = "2", pad_bytes_after = "1")]
    flags: u16,

    #[deku(bytes = "1")]
    penalty: u8,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    numstops: u8,
}
