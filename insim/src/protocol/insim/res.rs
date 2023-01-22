use deku::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use super::{PlayerFlags, RaceResultFlags};
use crate::protocol::identifiers::{PlayerId, RequestId};
use crate::string::{istring, CodepageString};
use crate::vehicle::Vehicle;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Race Result
pub struct Res {
    pub reqi: RequestId,

    pub plid: PlayerId,

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
