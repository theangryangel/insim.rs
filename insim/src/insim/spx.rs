use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_u32_duration, binrw_write_u32_duration},
    identifiers::{PlayerId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::{Fuel200, PenaltyInfo};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Split timing
pub struct Spx {
    pub reqi: RequestId,
    pub plid: PlayerId,

    #[br(parse_with = binrw_parse_u32_duration::<_>)]
    #[bw(write_with = binrw_write_u32_duration::<_>)]
    pub stime: Duration,
    #[br(parse_with = binrw_parse_u32_duration::<_>)]
    #[bw(write_with = binrw_write_u32_duration::<_>)]
    pub etime: Duration,

    pub split: u8,
    pub penalty: PenaltyInfo,

    pub numstops: u8,
    pub fuel200: Fuel200,
}
