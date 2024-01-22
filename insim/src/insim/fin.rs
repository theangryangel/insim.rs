use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
};

use crate::identifiers::{PlayerId, RequestId};

use bitflags::bitflags;

use super::PlayerFlags;

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    /// Race result confirmation flags
    pub struct RaceConfirmationFlags: u8 {
        /// Mentioned
        const MENTIONED = (1 << 0);
        /// Confirmed result
        const CONFIRMED = (1 << 1);
        /// Drive thru penalty
        const PENALTY_DT = (1 << 2);
        /// Stop-go penalty
        const PENALTY_SG = (1 << 3);
        /// 30 secs penalty
        const PENALTY_30 = (1 << 4);
        /// 45 secs penalty
        const PENALTY_45 = (1 << 5);
        /// Pit-stop was required
        const DID_NOT_PIT = (1 << 6);
    }
}

impl RaceConfirmationFlags {
    /// Was the player disqualified for any reason?
    pub fn disqualified(&self) -> bool {
        self.contains(RaceConfirmationFlags::PENALTY_DT)
            || self.contains(RaceConfirmationFlags::PENALTY_SG)
            || self.contains(RaceConfirmationFlags::DID_NOT_PIT)
    }

    /// Did the player receive a penalty for any reason?
    pub fn time_penalty(&self) -> bool {
        self.contains(RaceConfirmationFlags::PENALTY_30)
            || self.contains(RaceConfirmationFlags::PENALTY_45)
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Provisional finish notification: This is not a final result, you should use the [Res](super::Res) packet for this instead.
pub struct Fin {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique player id for this finish notification
    pub plid: PlayerId,

    #[br(parse_with = binrw_parse_duration::<u32, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u32, 1, _>)]
    /// Total time elapsed
    pub ttime: Duration,

    #[br(parse_with = binrw_parse_duration::<u32, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u32, 1, _>)]
    #[brw(pad_after = 1)]
    /// Best lap time
    pub btime: Duration,

    /// Total number of stops
    pub numstops: u8,

    #[brw(pad_after = 1)]
    /// Confirmation flags give extra context to the result
    pub confirm: RaceConfirmationFlags,

    /// Total laps completed
    pub lapsdone: u16,

    /// Player flags (help settings)
    pub flags: PlayerFlags,
}
