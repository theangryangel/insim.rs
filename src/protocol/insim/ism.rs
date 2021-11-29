use crate::string::InsimString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Ism {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "1", pad_bytes_after = "3")]
    pub host: u8,

    #[deku(bytes = "16")]
    pub hname: InsimString,
}
