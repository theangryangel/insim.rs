use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(
    InsimEncode, InsimDecode, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default,
)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
// *_VALID variation means this was cleared
pub enum PenaltyInfo {
    #[default]
    None = 0,
    DriveThru = 1,
    DriveThruValid = 2,
    StopGo = 3,
    StopGoValid = 4,
    Seconds30 = 5,
    Seconds45 = 6,
}

#[derive(Debug, Default, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum PenaltyReason {
    /// Unknown or cleared penalty
    #[default]
    Unknown = 0,

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

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Penalty
pub struct Pen {
    pub reqi: RequestId,
    pub plid: PlayerId,

    pub oldpen: PenaltyInfo,
    pub newpen: PenaltyInfo,

    #[insim(pad_bytes_after = "1")]
    pub reason: PenaltyReason,
}
