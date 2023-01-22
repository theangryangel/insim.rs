use insim_core::prelude::*;

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

    #[insim(
        reader = "istring::read(insim::rest, 24)",
        writer = "istring::write(insim::output, &self.uname, 24)"
    )]
    pub uname: String,

    #[insim(bytes = "24")]
    pub pname: CodepageString,

    #[insim(bytes = "8")]
    pub plate: CodepageString,

    pub cname: Vehicle,

    pub ttime: u32,

    #[insim(pad_bytes_after = "1")]
    pub btime: u32,

    pub numstops: u8,

    #[insim(pad_bytes_after = "1")]
    pub confirm: RaceResultFlags,

    pub lapsdone: u16,

    pub flags: PlayerFlags,

    pub resultnum: u8,

    pub numres: u8,

    pub pseconds: u16,
}
