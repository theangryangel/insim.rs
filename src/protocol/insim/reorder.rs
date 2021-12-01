use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Reorder {
    #[deku(bytes = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    nump: u8,

    #[deku(bytes = "1", count = "40")]
    plid: Vec<u8>,
}
