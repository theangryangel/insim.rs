use crate::string::InsimString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct PlayerRenamed {
    #[deku(bytes = "1")]
    reqi: u8,

    #[deku(bytes = "24")]
    pname: InsimString,

    #[deku(bytes = "8")]
    plate: InsimString,
}
