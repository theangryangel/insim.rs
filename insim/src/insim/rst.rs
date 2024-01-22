use insim_core::{
    binrw::{self, binrw},
    track::Track,
    wind::Wind,
};

use super::RaceLaps;
use crate::identifiers::RequestId;

use bitflags::bitflags;

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &HostFacts| x.bits())]
    /// Facts about a server, or race
    pub struct HostFacts: u16 {
        /// Can vote
        const CAN_VOTE = (1 << 0);
        /// Can select
        const CAN_SELECT = (1 << 1);
        /// Can joint mid-race
        const MID_RACE_JOIN = (1 << 5);
        /// Mandatory pit stop
        const MUST_PIT = (1 << 6);
        /// Car reset allowed
        const CAN_RESET = (1 << 7);
        /// Force cockpit view
        const FORCE_DRIVER_VIEW = (1 << 8);
        /// "Cruise" (no race, just free drive)
        const CRUISE = (1 << 9);
    }
}

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
    pub flags: HostFacts,
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
