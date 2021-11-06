use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Cnl {
    #[deku(bytes = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    ucid: u8,

    #[deku(bytes = "1")]
    reason: u8,

    #[deku(bytes = "1", pad_bytes_after = "2")]
    total: u8,
}
