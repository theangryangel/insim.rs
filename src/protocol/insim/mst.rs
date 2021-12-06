use crate::string::ICodepageString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Message Type - Send a message to LFS as if typed by a user
pub struct Mst {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "64")]
    pub msg: ICodepageString,
}
