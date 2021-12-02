use crate::string::InsimString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Auto X Info
pub struct Axi {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    pub axstart: u8,
    pub numcp: u8,
    pub numo: u16,

    #[deku(bytes = "32")]
    pub lname: InsimString,
}
