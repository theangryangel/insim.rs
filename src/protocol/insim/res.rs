use super::PlayerFlags;
use crate::string::{ICodepageString, IString, IVehicleString};
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Race Result
pub struct Res {
    pub reqi: u8,

    pub plid: u8,

    #[deku(bytes = "24")]
    pub uname: IString,

    #[deku(bytes = "24")]
    pub pname: ICodepageString,

    #[deku(bytes = "8")]
    pub plate: ICodepageString,

    pub cname: IVehicleString,

    pub ttime: u32,

    #[deku(pad_bytes_after = "1")]
    pub btime: u32,

    pub numstops: u8,

    #[deku(pad_bytes_after = "1")]
    pub confirm: u8,

    pub lapsdone: u16,

    pub flags: PlayerFlags,

    pub resultnum: u8,

    pub numres: u8,

    pub pseconds: u16,
}
