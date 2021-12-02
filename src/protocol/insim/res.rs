use super::PlayerFlags;
use crate::string::InsimString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Race Result
pub struct Res {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub plid: u8,

    #[deku(bytes = "24")]
    pub uname: InsimString,

    #[deku(bytes = "24")]
    pub pname: InsimString,

    #[deku(bytes = "8")]
    pub plate: InsimString,

    #[deku(bytes = "4")]
    pub cname: InsimString,

    #[deku(bytes = "4")]
    pub ttime: u32,

    #[deku(bytes = "4", pad_bytes_after = "1")]
    pub btime: u32,

    #[deku(bytes = "1")]
    pub numstops: u8,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub confirm: u8,

    #[deku(bytes = "2")]
    pub lapsdone: u16,

    #[deku(bytes = "2")]
    pub flags: PlayerFlags,

    #[deku(bytes = "1")]
    pub resultnum: u8,

    #[deku(bytes = "1")]
    pub numres: u8,

    #[deku(bytes = "2")]
    pub pseconds: u16,
}
