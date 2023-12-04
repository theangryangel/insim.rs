use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_u32_duration, binrw_write_u32_duration},
    identifiers::{PlayerId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

use super::PlayerFlags;

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    pub struct RaceResultFlags: u8 {
        const MENTIONED = (1 << 0);
        const CONFIRMED = (1 << 1);
        const PENALTY_DT = (1 << 2);
        const PENALTY_SG = (1 << 3);
        const PENALTY_30 = (1 << 4);
        const PENALTY_45 = (1 << 5);
        const NO_PIT = (1 << 6);
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

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Provisional finish notification: This is not a final result, you should use the [Res](super::Res) packet for this instead.
pub struct Fin {
    pub reqi: RequestId,
    pub plid: PlayerId,

    #[br(parse_with = binrw_parse_u32_duration::<_>)]
    #[bw(write_with = binrw_write_u32_duration::<_>)]
    pub ttime: Duration,

    #[br(parse_with = binrw_parse_u32_duration::<_>)]
    #[bw(write_with = binrw_write_u32_duration::<_>)]
    #[brw(pad_after = 1)]
    pub btime: Duration,

    pub numstops: u8,

    #[brw(pad_after = 1)]
    pub confirm: RaceResultFlags,

    pub lapsdone: u16,
    pub flags: PlayerFlags,
}
