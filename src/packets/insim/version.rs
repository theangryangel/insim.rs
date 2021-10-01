use crate::string::InsimString;
use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Version {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    reqi: u8,

    #[deku(bytes = "8")]
    version: InsimString,

    #[deku(bytes = "6")]
    product: InsimString,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    insimver: u16,
}
