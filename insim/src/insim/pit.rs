use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
    identifiers::{PlayerId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

use super::{Fuel, PenaltyInfo, PlayerFlags, TyreCompound};

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    pub struct PitStopWorkFlags: u32 {
         const NOTHING = 0;
         const STOP = (1 << 0);
         const FR_DAM = (1 << 1);
         const FR_WHL = (1 << 2);
         const PSE_LE_FR_DAM = (1 << 3);
         const PSE_LE_FR_WHL = (1 << 4);
         const PSE_RI_FR_DAM = (1 << 5);
         const PSE_RI_FR_WHL = (1 << 6);
         const PSE_RE_DAM = (1 << 7);
         const PSE_RE_WHL = (1 << 8);
         const PSE_LE_RE_DAM = (1 << 9);
         const PSE_LE_RE_WHL = (1 << 10);
         const PSE_RI_RE_DAM = (1 << 11);
         const PSE_RI_RE_WHL = (1 << 12);
         const PSE_BODY_MINOR = (1 << 13);
         const PSE_BODY_MAJOR = (1 << 14);
         const PSE_SETUP = (1 << 15);
         const PSE_REFUEL = (1 << 16);
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Pit stop (stop at the garage, not "tele-pit")
pub struct Pit {
    pub reqi: RequestId,
    pub plid: PlayerId,

    pub lapsdone: u16,
    pub flags: PlayerFlags,

    pub fueladd: Fuel,
    pub penalty: PenaltyInfo,
    #[brw(pad_after = 1)]
    pub numstops: u8,

    pub tyres: [TyreCompound; 4],
    #[brw(pad_after = 4)]
    pub work: PitStopWorkFlags,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Pit Stop Finished
pub struct Psf {
    pub reqi: RequestId,
    pub plid: PlayerId,

    #[brw(pad_after = 4)]
    #[br(parse_with = binrw_parse_duration::<u32, _>)]
    #[bw(write_with = binrw_write_duration::<u32, _>)]
    pub stime: Duration,
}

#[binrw]
#[derive(Debug, Default, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum PitLaneFact {
    #[default]
    /// Left pitlane
    Exit = 0,

    /// Entered pitlane
    Enter = 1,

    /// Entered for no known reason
    EnterNoPurpose = 2,

    /// Entered for Drive-through penalty
    EnterDriveThru = 3,

    /// Entered for a stop-go (time) penalty
    EnterStopGo = 4,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// PitLane
pub struct Pla {
    pub reqi: RequestId,
    pub plid: PlayerId,

    #[brw(pad_after = 3)]
    pub fact: PitLaneFact,
}

impl Pla {
    pub fn entered(&self) -> bool {
        self.fact != PitLaneFact::Exit
    }

    pub fn exited(&self) -> bool {
        !self.entered()
    }
}
