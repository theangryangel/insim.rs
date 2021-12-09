use super::PlayerFlags;
use crate::packet_flags;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

packet_flags! {
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct RaceResultFlags: u8 {
        MENTIONED => (1 << 0),
        CONFIRMED => (1 << 1),
        PENALTY_DT => (1 << 2),
        PENALTY_SG => (1 << 3),
        PENALTY_30 => (1 << 4),
        PENALTY_45 => (1 << 5),
        NO_PIT => (1 << 6),
    }
}

impl RaceResultFlags {
    /// Was the player disqualified for any reason?
    pub fn disqualified(&self) -> bool {
        self.contains(RaceResultFlags::PENALTY_DT)
            || self.contains(RaceResultFlags::PENALTY_SG)
            || self.contains(RaceResultFlags::NO_PIT)
    }

    /// Did the player receive a penalty for any reason?
    pub fn time_penalty(&self) -> bool {
        self.contains(RaceResultFlags::PENALTY_30) || self.contains(RaceResultFlags::PENALTY_45)
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Provisional finish notification: This is not a final result, you should use the [Res](super::Res) packet for this instead.
pub struct Fin {
    pub reqi: u8,

    pub plid: u8,

    pub ttime: u32,

    #[deku(pad_bytes_after = "1")]
    pub btime: u32,

    pub numstops: u8,

    #[deku(pad_bytes_after = "1")]
    pub confirm: RaceResultFlags,

    pub lapsdone: u16,

    pub flags: PlayerFlags,
}
