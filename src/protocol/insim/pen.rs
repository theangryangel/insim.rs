use crate::packet_flags;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

packet_flags! {
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

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(type = "u8", endian = "little")]
pub enum PenaltyReason {
    /// Unknown or cleared penalty
    #[deku(id = "0")]
    None,

    /// Penalty given by admin
    #[deku(id = "1")]
    Admin,

    /// Driving wrong way
    #[deku(id = "2")]
    WrongWay,

    /// False start
    #[deku(id = "3")]
    FalseStart,

    /// Speeding in pit lane
    #[deku(id = "4")]
    Speeding,

    /// Stop-go in pit stop too short
    #[deku(id = "5")]
    StopShort,

    /// Compulsory stop is too late
    #[deku(id = "6")]
    StopLate,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Penalty
pub struct Pen {
    pub reqi: u8,

    pub plid: u8,

    pub oldpen: PenaltyInfo,

    pub newpen: PenaltyInfo,

    #[deku(pad_bytes_after = "1")]
    pub reason: PenaltyReason,
}
