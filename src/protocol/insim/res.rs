use super::{PlayerFlags, RaceResultFlags};
use crate::string::{istring, CodepageString};
use crate::vehicle::Vehicle;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Race Result
pub struct Res {
    pub reqi: u8,

    pub plid: u8,

    #[deku(
        reader = "istring::read(deku::rest, 24)",
        writer = "istring::write(deku::output, &self.uname, 24)"
    )]
    pub uname: String,

    #[deku(bytes = "24")]
    pub pname: CodepageString,

    #[deku(bytes = "8")]
    pub plate: CodepageString,

    pub cname: Vehicle,

    pub ttime: u32,

    #[deku(pad_bytes_after = "1")]
    pub btime: u32,

    pub numstops: u8,

    #[deku(pad_bytes_after = "1")]
    pub confirm: RaceResultFlags,

    pub lapsdone: u16,

    pub flags: PlayerFlags,

    pub resultnum: u8,

    pub numres: u8,

    pub pseconds: u16,
}
