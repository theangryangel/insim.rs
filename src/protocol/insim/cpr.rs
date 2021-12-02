use crate::string::InsimString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Connection Player Renamed
pub struct Cpr {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "24")]
    pub pname: InsimString,

    #[deku(bytes = "8")]
    pub plate: InsimString,
}
