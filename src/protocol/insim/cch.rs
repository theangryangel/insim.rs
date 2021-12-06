use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
// Camera Change
pub struct Cch {
    pub reqi: u8,

    pub plid: u8,

    #[deku(pad_bytes_after = "3")]
    pub camera: u8,
}
