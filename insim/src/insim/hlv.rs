use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
    identifiers::{PlayerId, RequestId},
};
use std::time::Duration;

#[cfg(feature = "serde")]
use serde::Serialize;

use super::CarContact;

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
/// Used within [Hlv] to indicate the hotlap validity failure reason.
pub enum Hlvc {
    #[default]
    Ground = 0,

    Wall = 1,

    Speeding = 4,

    OutOfBounds = 5,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Reports incidents that would violate Hot Lap Validity checks.
pub struct Hlv {
    pub reqi: RequestId,
    pub plid: PlayerId,

    #[brw(pad_after = 1)]
    pub hlvc: Hlvc,

    #[br(parse_with = binrw_parse_duration::<u16, 10, _>)]
    #[bw(write_with = binrw_write_duration::<u16, 10, _>)]
    pub time: Duration,
    pub c: CarContact,
}
