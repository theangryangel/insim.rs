use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
    identifiers::{PlayerId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::CarContact;

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
/// Used within the [Csc] packet to indicate the type of state change.
pub enum CscAction {
    #[default]
    Stop = 0,

    Start = 1,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Car State Changed
pub struct Csc {
    pub reqi: RequestId,

    #[brw(pad_after = 1)]
    pub plid: PlayerId,

    #[brw(pad_after = 2)]
    pub action: CscAction,

    #[br(parse_with = binrw_parse_duration::<u32, _>)]
    #[bw(write_with = binrw_write_duration::<u32, _>)]
    pub time: Duration,

    pub c: CarContact,
}
