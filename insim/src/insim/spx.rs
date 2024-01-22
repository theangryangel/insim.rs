use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
};

use crate::identifiers::{PlayerId, RequestId};

use super::{Fuel200, PenaltyInfo};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Split X timing
pub struct Spx {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique player id for this the split timing
    pub plid: PlayerId,

    #[br(parse_with = binrw_parse_duration::<u32, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u32, 1, _>)]
    /// Split duration
    pub stime: Duration,

    #[br(parse_with = binrw_parse_duration::<u32, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u32, 1, _>)]
    // Total elapsed time
    pub etime: Duration,

    /// Split number
    pub split: u8,

    /// Any penalties the user has received
    pub penalty: PenaltyInfo,

    /// The number of stops taken
    pub numstops: u8,

    /// When /showfuel yes: double fuel percent / no: 255
    pub fuel200: Fuel200,
}
