use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct TakeOverCar {
    #[deku(bytes = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    plid: u8,

    #[deku(bytes = "1")]
    olducid: u8,

    #[deku(bytes = "1", pad_bytes_after = "2")]
    newucid: u8,
}
