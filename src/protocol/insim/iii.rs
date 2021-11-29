use crate::string::InsimString;
use deku::ctx::Size;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Iii {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub ucid: u8,

    #[deku(bytes = "1", pad_bytes_after = "2")]
    pub plid: u8,

    #[deku(reader = "InsimString::read(deku::rest, Size::Bytes(deku::rest.len() / 8))")]
    pub msg: InsimString,
}
