use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Race Flag
pub struct Flg {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub plid: u8,

    #[deku(bytes = "1")]
    pub offon: u8,

    #[deku(bytes = "1")]
    pub flag: u8,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub carbehind: u8,
}
