use bitflags::bitflags;
use insim_core::{track::Track, wind::Wind};

use super::RaceLaps;
use crate::identifiers::RequestId;

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    /// Facts about a server, or race
    pub struct RaceFlags: u16 {
        /// Can vote
        const CAN_VOTE = (1 << 0);
        /// Can select
        const CAN_SELECT = (1 << 1);
        /// Can joint mid-race
        const MID_RACE = (1 << 5);
        /// Mandatory pit stop
        const MUST_PIT = (1 << 6);
        /// Car reset allowed
        const CAN_RESET = (1 << 7);
        /// Force cockpit view
        const FCV = (1 << 8);
        /// "Cruise" (no race, just free drive)
        const CRUISE = (1 << 9);
    }
}

generate_bitflag_helpers!(
    RaceFlags,
    pub can_vote => CAN_VOTE,
    pub can_select => CAN_SELECT,
    pub can_mid_race_join => MID_RACE,
    pub can_reset => CAN_RESET,
    pub forces_cockpit_view => FCV,
    pub is_cruise => CRUISE
);

impl_bitflags_from_to_bytes!(RaceFlags, u16);

#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Race Start - informational - sent when a race starts
pub struct Rst {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[read_write_buf(pad_after = 1)]
    pub reqi: RequestId,

    /// Total number of race laps
    pub racelaps: RaceLaps,

    /// Qualifying minutes, 0 if racing
    pub qualmins: u8,

    /// Total number of players
    pub nump: u8,

    /// Lap timing
    // TODO: needs decoding and strongly typing
    pub timing: u8,

    /// The track
    pub track: Track,

    /// The weather
    pub weather: u8,

    /// The wind conditions
    pub wind: Wind,

    /// The race/host facts (i.e. can pit, etc.)
    pub flags: RaceFlags,

    /// Total number of nodes in the path
    pub numnodes: u16,

    /// The index of the finish node
    pub finish: u16,

    /// The index of the split1 node
    pub split1: u16,

    /// The index of the split2 node
    pub split2: u16,

    /// The index of the split3 node
    pub split3: u16,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rst() {
        assert_from_to_bytes!(
            Rst,
            [
                1,  // reqi
                0,  // zero
                10, // racelaps
                45, // qualmins
                12, // nump
                66, // timing
                b'B', b'L', b'1', 0, 0, 0,   // track
                2,   // weather
                2,   // wind
                2,   // flags (1)
                1,   // flags (2)
                146, // numnodes (1)
                1,   // numnodes (2)
                109, // finish (1)
                1,   // finish (2)
                95,  // split1 (1)
                0,   // split1 (2)
                255, // split2 (1)
                0,   // split2 (2)
                255, // split3 (1)
                255, // split3 (2)
            ],
            |parsed: Rst| {
                assert_eq!(parsed.reqi, RequestId(1));
                assert_eq!(parsed.nump, 12);
                assert_eq!(parsed.numnodes, 402);
                assert!(matches!(parsed.racelaps, RaceLaps::Laps(10)));
                assert_eq!(parsed.qualmins, 45);
                assert!(matches!(parsed.wind, Wind::Strong));
            }
        )
    }
}
