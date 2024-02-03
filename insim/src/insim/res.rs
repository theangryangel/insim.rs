use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
    vehicle::Vehicle,
};

use super::{PlayerFlags, RaceConfirmFlags};
use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Race Result - qualifying or confirmed result
pub struct Res {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// The unique player ID that this race result is for
    pub plid: PlayerId,

    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    /// The LFS.net username of the player
    pub uname: String,

    #[bw(write_with = binrw_write_codepage_string::<24, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<24, _>)]
    /// The name of the player
    pub pname: String,

    #[bw(write_with = binrw_write_codepage_string::<8, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<8, _>)]
    /// The number plate of the player
    pub plate: String,

    /// The vehicle they finished in
    pub cname: Vehicle,

    #[br(parse_with = binrw_parse_duration::<u32, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u32, 1, _>)]
    /// The total time
    pub ttime: Duration,

    #[brw(pad_after = 1)]
    #[br(parse_with = binrw_parse_duration::<u32, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u32, 1, _>)]
    /// The best lap time
    pub btime: Duration,

    /// The number of pit stops taken
    pub numstops: u8,

    /// The result flags. Where they DNF?
    #[brw(pad_after = 1)]
    pub confirm: RaceConfirmFlags,

    /// The number of laps done
    pub lapsdone: u16,

    /// Additional information about the player.
    pub flags: PlayerFlags,

    /// Finish or qualify pos (0 = win / 255 = not added to table)
    pub resultnum: u8,

    /// Total number of results (qualify doesn't always add a new one)
    pub numres: u8,

    /// Penalty time in seconds (already included in race time)
    pub pseconds: u16,
}
