use crate::string::InsimString;
use deku::ctx::Size;
use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct MessageOut {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub ucid: u8,

    #[deku(bytes = "1")]
    pub plid: u8,

    #[deku(bytes = "1")]
    pub usertype: u8,

    #[deku(bytes = "1")]
    pub textstart: u8,

    #[deku(reader = "InsimString::read(deku::rest, Size::Bytes(deku::rest.len() / 8))")]
    pub msg: InsimString,
}
