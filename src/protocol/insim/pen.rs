use crate::packet_flags;
use deku::prelude::*;
use serde::Serialize;

packet_flags! {
    // *_VALID variation means this was cleared
    #[derive(Serialize)]
    pub struct PenaltyInfo: u8 {
        DRIVE_THRU => (1 << 0),
        DRIVE_THRU_VALID => (1 << 1),
        STOP_GO => (1 << 2),
        STOP_GO_VALID => (1 << 3),
        SECS_30 => (1 << 4),
        SECS_45 => (1 << 5),
    }
}

packet_flags! {
    #[derive(Serialize)]
    pub struct PenaltyReason: u8 {
        ADMIN => (1 << 0),
        WRONG_WAY => (1 << 1),
        FALSE_START => (1 << 2),
        SPEEDING => (1 << 3),
        STOP_SHORT => (1 << 4),
        STOP_LATE => (1 << 5),
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Penalty
pub struct Pen {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub plid: u8,

    #[deku(bytes = "1")]
    pub oldpen: PenaltyInfo,

    #[deku(bytes = "1")]
    pub newpen: PenaltyInfo,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reason: PenaltyReason,
}
