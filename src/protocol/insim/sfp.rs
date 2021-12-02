use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// State Flags Pack
pub struct Sfp {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "2")]
    pub flag: u16,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub onoff: u8,
}
