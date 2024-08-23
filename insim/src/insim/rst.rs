use bitflags::bitflags;
use insim_core::{
    binrw::{self, binrw},
    track::Track,
    wind::Wind,
};

use super::RaceLaps;
use crate::identifiers::RequestId;

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &RaceFlags| x.bits())]
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

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Race Start - informational - sent when a race starts
pub struct Rst {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// Total number of race laps
    pub racelaps: RaceLaps,

    /// Qualifying minutes, 0 if racing
    pub qualmins: u8,

    /// Total number of players
    pub nump: u8,

    /// Lap timing
    pub timing: u8, // TODO - needs decoding and strongly typing

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
