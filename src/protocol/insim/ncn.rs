use crate::string::InsimString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Ncn {
    #[deku(bytes = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    ucid: u8,

    #[deku(bytes = "24")]
    uname: InsimString,

    #[deku(bytes = "24")]
    pname: InsimString,

    #[deku(bytes = "1")]
    admin: u8,

    #[deku(bytes = "1")]
    total: u8,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    flags: u8,
}
