use std::time::Duration;

use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
    vehicle::Vehicle,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::{PlayerFlags, RaceResultFlags};

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Race Result
pub struct Res {
    pub reqi: RequestId,
    pub plid: PlayerId,

    #[insim(bytes = "24")]
    pub uname: String,
    #[insim(bytes = "24")]
    pub pname: String,
    #[insim(bytes = "8")]
    pub plate: String,
    pub cname: Vehicle,

    pub ttime: Duration,
    #[insim(pad_bytes_after = "1")]
    pub btime: Duration,

    pub numstops: u8,
    #[insim(pad_bytes_after = "1")]
    pub confirm: RaceResultFlags,

    pub lapsdone: u16,
    pub flags: PlayerFlags,

    pub resultnum: u8,
    pub numres: u8,
    pub pseconds: u16,
}
