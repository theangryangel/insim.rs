use crate::string::IString;
use deku::ctx::Size;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Message to Connection - Send a message to a specific connection, restricted to hosts only
pub struct Mtc {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub sound: u8,

    #[deku(bytes = "1")]
    pub ucid: u8,

    #[deku(bytes = "1", pad_bytes_after = "2")]
    pub plid: u8,

    #[deku(reader = "IString::read(deku::rest, Size::Bytes(deku::rest.len() / 8))")]
    pub msg: IString,
}
