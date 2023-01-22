use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{
    protocol::identifiers::{PlayerId, RequestId},
};

use bitflags::bitflags;

bitflags! {
    // *_VALID variation means this was cleared
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct PenaltyInfo: u8 {
        DRIVE_THRU => (1 << 0),
        DRIVE_THRU_VALID => (1 << 1),
        STOP_GO => (1 << 2),
        STOP_GO_VALID => (1 << 3),
        SECS_30 => (1 << 4),
        SECS_45 => (1 << 5),
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum PenaltyReason {
    /// Unknown or cleared penalty
    None = 0,

    /// Penalty given by admin
    Admin = 1,

    /// Driving wrong way
    WrongWay = 2,

    /// False start
    FalseStart = 3,

    /// Speeding in pit lane
    Speeding = 4,

    /// Stop-go in pit stop too short
    StopShort = 5,

    /// Compulsory stop is too late
    StopLate = 6,
}

impl Default for PenaltyReason {
    fn default() -> Self {
        PenaltyReason::None
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Penalty
pub struct Pen {
    pub reqi: RequestId,

    pub plid: PlayerId,

    pub oldpen: PenaltyInfo,

    pub newpen: PenaltyInfo,

    #[deku(pad_bytes_after = "1")]
    pub reason: PenaltyReason,
}
