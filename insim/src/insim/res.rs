use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
    identifiers::{PlayerId, RequestId},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
    vehicle::Vehicle,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::{PlayerFlags, RaceResultFlags};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Race Result
pub struct Res {
    pub reqi: RequestId,
    pub plid: PlayerId,

    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    pub uname: String,

    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    pub pname: String,

    #[bw(write_with = binrw_write_codepage_string::<8, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<8, _>)]
    pub plate: String,
    pub cname: Vehicle,

    #[br(parse_with = binrw_parse_duration::<u32, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u32, 1, _>)]
    pub ttime: Duration,

    #[brw(pad_after = 1)]
    #[br(parse_with = binrw_parse_duration::<u32, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u32, 1, _>)]
    pub btime: Duration,

    pub numstops: u8,
    #[brw(pad_after = 1)]
    pub confirm: RaceResultFlags,

    pub lapsdone: u16,
    pub flags: PlayerFlags,

    pub resultnum: u8,
    pub numres: u8,
    pub pseconds: u16,
}
