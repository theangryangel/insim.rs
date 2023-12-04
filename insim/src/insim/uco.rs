use std::time::Duration;

use insim_core::{
    identifiers::{PlayerId, RequestId},
    binrw::{self, binrw},
    duration::{binrw_write_u32_duration, binrw_parse_u32_duration},
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::{CarContact, ObjectInfo};

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum UcoAction {
    #[default]
    CircleEnter = 0, // entered a circle

    CircleLeave = 1, // left a circle

    CpFwd = 2, // crossed cp in forward direction

    CpRev = 3,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// User Control Object
pub struct Uco {
    pub reqi: RequestId,
    #[brw(pad_after = 1)]
    pub plid: PlayerId,

    #[brw(pad_after = 2)]
    pub action: UcoAction,

    #[br(parse_with = binrw_parse_u32_duration::<_>)]
    #[bw(write_with = binrw_write_u32_duration::<_>)]
    pub time: Duration,

    pub c: CarContact,

    pub info: ObjectInfo,
}
