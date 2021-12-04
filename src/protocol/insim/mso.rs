use crate::string::IString;
use deku::ctx::Size;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Message Out
pub struct Mso {
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

    #[deku(reader = "IString::read(deku::rest, Size::Bytes(deku::rest.len() / 8))")]
    pub msg: IString,
}
