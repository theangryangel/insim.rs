use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
};

use crate::identifiers::{PlayerId, RequestId};

use bitflags::bitflags;

use super::{Fuel, PenaltyInfo, PlayerFlags, TyreCompound};

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    /// Work which was carried out at a pitstop. Used in [Pit].
    pub struct PitStopWorkFlags: u32 {
        /// Nothing.
        const NOTHING = 0;
        /// Stop
        const STOP = (1 << 0);
        /// Front damage
        const FR_DAM = (1 << 1);
        // FR wheel
        const FR_WHL = (1 << 2);
        /// ???
        const PSE_LE_FR_DAM = (1 << 3);
        /// ???
        const PSE_LE_FR_WHL = (1 << 4);
        /// ???
        const PSE_RI_FR_DAM = (1 << 5);
        /// ???
        const PSE_RI_FR_WHL = (1 << 6);
        /// ???
        const PSE_RE_DAM = (1 << 7);
        /// ???
        const PSE_RE_WHL = (1 << 8);
        /// ???
        const PSE_LE_RE_DAM = (1 << 9);
        /// ???
        const PSE_LE_RE_WHL = (1 << 10);
        /// ???
        const PSE_RI_RE_DAM = (1 << 11);
        /// ???
        const PSE_RI_RE_WHL = (1 << 12);
        /// ???
        const PSE_BODY_MINOR = (1 << 13);
        /// ???
        const PSE_BODY_MAJOR = (1 << 14);
        /// ???
        const PSE_SETUP = (1 << 15);
        /// Refuel
        const PSE_REFUEL = (1 << 16);
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Pit stop (stop at the garage, not "tele-pit")
pub struct Pit {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Players unique ID
    pub plid: PlayerId,

    /// Laps completed
    pub lapsdone: u16,

    /// Player Flags. See [PlayerFlags].
    pub flags: PlayerFlags,

    /// When /showfuel yes: fuel added percent / no: 255
    pub fueladd: Fuel,

    /// Any penalties that were cleared
    pub penalty: PenaltyInfo,

    /// Total number of stops
    #[brw(pad_after = 1)]
    pub numstops: u8,

    /// Tyres!
    pub tyres: [TyreCompound; 4],

    /// What work was carried out?
    #[brw(pad_after = 4)]
    pub work: PitStopWorkFlags,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Pit Stop Finished
pub struct Psf {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Player's unique ID
    pub plid: PlayerId,

    #[brw(pad_after = 4)]
    #[br(parse_with = binrw_parse_duration::<u32, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u32, 1, _>)]
    /// How long were they pitting for?
    pub stime: Duration,
}

#[binrw]
#[derive(Debug, Default, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
/// Pit lane fact, or info. Used in [Pla].
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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// PitLane
pub struct Pla {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Player's unique ID
    pub plid: PlayerId,

    /// Fact
    #[brw(pad_after = 3)]
    pub fact: PitLaneFact,
}

impl Pla {
    /// Did the player enter the pitlate?
    pub fn entered_pitlane(&self) -> bool {
        self.fact != PitLaneFact::Exit
    }

    /// Did the player exit the pitlane?
    pub fn exited_pitlane(&self) -> bool {
        !self.entered_pitlane()
    }
}
