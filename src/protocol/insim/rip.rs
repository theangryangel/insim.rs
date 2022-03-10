use crate::string::istring;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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

impl Default for RipError {
    fn default() -> Self {
        RipError::Ok
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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

    #[deku(
        reader = "istring::read(deku::rest, 64)",
        writer = "istring::write(deku::output, &self.rname, 64)"
    )]
    pub rname: String,
}
