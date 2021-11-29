use crate::string::InsimString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Mst {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "64")]
    pub msg: InsimString,
}
