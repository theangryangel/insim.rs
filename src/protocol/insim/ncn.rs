use crate::string::IString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// New Connection
pub struct Ncn {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub ucid: u8,

    #[deku(bytes = "24")]
    pub uname: IString,

    #[deku(bytes = "24")]
    pub pname: IString,

    #[deku(bytes = "1")]
    pub admin: u8,

    #[deku(bytes = "1")]
    pub total: u8,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub flags: u8,
}
