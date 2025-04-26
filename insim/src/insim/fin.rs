use std::time::Duration;

use super::PlayerFlags;
use crate::identifiers::{PlayerId, RequestId};

bitflags::bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    /// Race result confirmation flags
    pub struct RaceConfirmFlags: u8 {
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

impl_bitflags_from_to_bytes!(RaceConfirmFlags, u8);

generate_bitflag_helpers! {
    RaceConfirmFlags,

    pub is_mentioned => MENTIONED,
    pub is_confirmed_result => CONFIRMED,
    pub has_drive_thru_penalty => PENALTY_DT,
    pub has_stop_go_penalty => PENALTY_SG,
    pub has_30s_penalty => PENALTY_30,
    pub has_45s_penalty => PENALTY_45,
    pub skipped_mandatory_pit_stop => DID_NOT_PIT
}

impl RaceConfirmFlags {
    /// Was the player disqualified for any reason?
    pub fn is_disqualified(&self) -> bool {
        self.contains(RaceConfirmFlags::PENALTY_DT)
            || self.contains(RaceConfirmFlags::PENALTY_SG)
            || self.contains(RaceConfirmFlags::DID_NOT_PIT)
    }

    /// Was the player disqualified for any reason?
    #[deprecated = "Prefer is_disqualified"]
    pub fn disqualified(&self) -> bool {
        self.is_disqualified()
    }

    /// Did the player receive any time penalties?
    pub fn has_time_penalty(&self) -> bool {
        self.contains(RaceConfirmFlags::PENALTY_30) || self.contains(RaceConfirmFlags::PENALTY_45)
    }

    /// Did the player receive any time penalties?
    #[deprecated = "Prefer has_time_penalty"]
    pub fn time_penalty(&self) -> bool {
        self.has_time_penalty()
    }
}

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Provisional finish notification: This is not a final result, you should use the [Res](super::Res) packet for this instead.
pub struct Fin {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique player id for this finish notification
    pub plid: PlayerId,

    #[read_write_buf(duration(milliseconds = u32))]
    /// Total time elapsed
    pub ttime: Duration,

    #[read_write_buf(duration(milliseconds = u32), pad_after = 1)]
    /// Best lap time
    pub btime: Duration,

    /// Total number of stops
    pub numstops: u8,

    #[read_write_buf(pad_after = 1)]
    /// Confirmation flags give extra context to the result
    pub confirm: RaceConfirmFlags,

    /// Total laps completed
    pub lapsdone: u16,

    /// Player flags (help settings)
    pub flags: PlayerFlags,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fin() {
        assert_from_to_bytes!(
            Fin,
            [
                0,   // reqi
                3,   // plid
                145, // ttime (1)
                4,   // ttime (2)
                2,   // ttime (3)
                0,   // ttime (4)
                65,  // btime (1)
                56,  // btime (2)
                0,   // btime (3)
                0,   // btime (4)
                0,   // spa
                1,   // numstops
                22,  // confirm
                0,   // spb
                68,  // lapsdone (1)
                0,   // lapsdone (2)
                9,   // flags (1)
                0,   // flags (2)
            ],
            |fin: Fin| {
                assert_eq!(fin.ttime, Duration::from_millis(132241));
                assert_eq!(fin.btime, Duration::from_millis(14401));
            }
        );
    }
}
