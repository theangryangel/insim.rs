use insim_core::prelude::*;
use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

use super::PlayerFlags;
use crate::{
    protocol::identifiers::{PlayerId, RequestId},
};

bitflags! {
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

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Provisional finish notification: This is not a final result, you should use the [Res](super::Res) packet for this instead.
pub struct Fin {
    pub reqi: RequestId,

    pub plid: PlayerId,

    pub ttime: u32,

    #[insim(pad_bytes_after = "1")]
    pub btime: u32,

    pub numstops: u8,

    #[insim(pad_bytes_after = "1")]
    pub confirm: RaceResultFlags,

    pub lapsdone: u16,

    pub flags: PlayerFlags,
}
