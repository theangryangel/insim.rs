use insim_core::{
    binrw::{self, binrw},
    identifiers::RequestId,
    racelaps::RaceLaps,
    track::Track,
    wind::Wind,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &HostFacts| x.bits())]
    pub struct HostFacts: u16 {
         const CAN_VOTE = (1 << 0);
         const CAN_SELECT = (1 << 1);
         const MID_RACE_JOIN = (1 << 5);
         const MUST_PIT = (1 << 6);
         const CAN_RESET = (1 << 7);
         const FORCE_DRIVER_VIEW = (1 << 8);
         const CRUISE = (1 << 9);
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Race Start
pub struct Rst {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    pub racelaps: RaceLaps,
    /// Qualifying minutes, 0 if racing
    pub qualmins: u8,

    pub nump: u8,
    pub timing: u8,

    pub track: Track,
    pub weather: u8,
    pub wind: Wind,

    pub flags: HostFacts,
    pub numnodes: u16,
    pub finish: u16,
    pub split1: u16,
    pub split2: u16,
    pub split3: u16,
}
