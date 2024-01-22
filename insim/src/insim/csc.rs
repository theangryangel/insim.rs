use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
};

use crate::identifiers::{PlayerId, RequestId};

use super::CarContact;

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
/// Used within the [Csc] packet to indicate the type of state change.
pub enum CscAction {
    #[default]
    /// Stopped
    Stop = 0,

    /// Started
    Start = 1,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Car State Changed - reports a change in a car's state (currently start or stop)
pub struct Csc {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique player ID
    #[brw(pad_after = 1)]
    pub plid: PlayerId,

    /// Action that was taken
    #[brw(pad_after = 2)]
    pub action: CscAction,

    /// Time since start (warning: this is looping)
    #[br(parse_with = binrw_parse_duration::<u32, 10, _>)]
    #[bw(write_with = binrw_write_duration::<u32, 10, _>)]
    pub time: Duration,

    /// Any contact that may have happened
    pub c: CarContact,
}
