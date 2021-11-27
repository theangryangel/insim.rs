use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Sfp {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    reqi: u8,

    #[deku(bytes = "2")]
    flag: u16,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    onoff: u8,
}
