use insim_core::binrw::{self, binrw};

use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
#[non_exhaustive]
/// Penalty types
pub enum PenaltyInfo {
    /// None, or cleared
    #[default]
    None = 0,

    /// Drive through
    Dt = 1,

    /// Drive Through completed
    DtValid = 2,

    /// Stop go
    Sg = 3,

    /// Stop go completed
    SgValid = 4,

    /// 30 Seconds added
    Seconds30 = 5,

    /// 45 seconds added
    Seconds45 = 6,
}

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
/// Enum of reasons for a penalty being applied to a player
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

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Penalty received or cleared by player
pub struct Pen {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique player id which changed
    pub plid: PlayerId,

    /// The original penalty state
    pub oldpen: PenaltyInfo,

    /// The new penalty state
    pub newpen: PenaltyInfo,

    /// The reason for the change
    #[brw(pad_after = 1)]
    pub reason: PenaltyReason,
}
