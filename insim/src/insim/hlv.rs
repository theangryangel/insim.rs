use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
};

use super::CarContact;
use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
/// Used within [Hlv] to indicate the hotlap validity failure reason.
pub enum Hlvc {
    /// Ground
    #[default]
    Ground = 0,

    /// Wall
    Wall = 1,

    /// Speeding in pitlane
    Speeding = 4,

    /// Out of bounds
    OutOfBounds = 5,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Reports incidents that would violate Hot Lap Validity checks.
pub struct Hlv {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique ID of player
    pub plid: PlayerId,

    /// How did we invalidate this hotlap? See [Hlvc].
    #[brw(pad_after = 1)]
    pub hlvc: Hlvc,

    #[br(parse_with = binrw_parse_duration::<u16, 10, _>)]
    #[bw(write_with = binrw_write_duration::<u16, 10, _>)]
    /// When the violation occured. Warning: this is looping.
    pub time: Duration,

    /// Additional contact information. See [CarContact].
    pub c: CarContact,
}
