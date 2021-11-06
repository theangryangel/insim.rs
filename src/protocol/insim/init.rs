use crate::string::InsimString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Init {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    // we do not support this feature, using pad_bytes_before
    // on flags to mask it.
    //#[deku(bytes = "2")]
    //pub udpport: u16,
    #[deku(bytes = "2", pad_bytes_before = "2")]
    pub flags: u16,

    #[deku(bytes = "1")]
    pub version: u8,

    #[deku(bytes = "1")]
    pub prefix: u8,

    #[deku(bytes = "2")]
    pub interval: u16,

    #[deku(bytes = "16")]
    pub password: InsimString,

    #[deku(bytes = "16")]
    pub name: InsimString,
}
