use crate::string::InsimString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(endian = "little", type = "u8")]
pub enum RipError {
    #[deku(id = "0")]
    Ok,

    #[deku(id = "1")]
    Already,

    #[deku(id = "2")]
    Dedicated,

    #[deku(id = "3")]
    WrongMode,

    #[deku(id = "4")]
    NotReplay,

    #[deku(id = "5")]
    Corrupted,

    #[deku(id = "6")]
    NotFound,

    #[deku(id = "7")]
    Unloadable,

    #[deku(id = "8")]
    DestOOB,

    #[deku(id = "9")]
    Unknown,

    #[deku(id = "10")]
    User,

    #[deku(id = "11")]
    OOS,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Replay Information
pub struct Rip {
    pub reqi: u8,
    pub error: RipError,

    pub mpr: u8,
    pub paused: u8,
    #[deku(pad_bytes_after = "1")]
    pub options: u8, // FIXME: implement flags

    pub ctime: u32,
    pub ttime: u32,

    #[deku(bytes = "64")]
    pub rname: InsimString,
}
