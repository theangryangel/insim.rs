use crate::string::InsimString;
use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Version {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "8")]
    pub version: InsimString,

    #[deku(bytes = "6")]
    pub product: InsimString,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub insimver: u8,
}
