use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Sch {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    charb: u8,

    #[deku(bytes = "1", pad_bytes_after = "2")]
    flags: u8,
}
