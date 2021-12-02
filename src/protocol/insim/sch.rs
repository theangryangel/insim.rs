use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Send Single Character
pub struct Sch {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub charb: u8,

    #[deku(bytes = "1", pad_bytes_after = "2")]
    pub flags: u8, // bit 0: SHIFT / bit 1: CTRL
}
