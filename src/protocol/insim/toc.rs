use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Take Over Car
pub struct Toc {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub plid: u8,

    #[deku(bytes = "1")]
    pub olducid: u8,

    #[deku(bytes = "1", pad_bytes_after = "2")]
    pub newucid: u8,
}
