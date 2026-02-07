use std::time::Duration;

use bitflags::bitflags;

use super::{Fuel, PenaltyInfo, PlayerFlags, TyreCompound};
use crate::identifiers::{PlayerId, RequestId};

bitflags! {
    /// Work carried out during a pit stop.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct PitStopWorkFlags: u32 {
        /// No work carried out.
        const NOTHING = 0;
        /// Stop only
        const STOP = (1 << 0);
        /// Front damage
        const FR_DAM = (1 << 1);
        /// FR wheel
        const FR_WHL = (1 << 2);
        /// LE_FR_DAM
        const PSE_LE_FR_DAM = (1 << 3);
        /// LE_FR_WHL
        const PSE_LE_FR_WHL = (1 << 4);
        /// RI_FR_DAM
        const PSE_RI_FR_DAM = (1 << 5);
        /// RI_FR_WHL
        const PSE_RI_FR_WHL = (1 << 6);
        /// RE_DAM
        const PSE_RE_DAM = (1 << 7);
        /// RE_WHL
        const PSE_RE_WHL = (1 << 8);
        /// LE_RE_DAM
        const PSE_LE_RE_DAM = (1 << 9);
        /// LE_RE_WHL
        const PSE_LE_RE_WHL = (1 << 10);
        /// RI_RE_DAM
        const PSE_RI_RE_DAM = (1 << 11);
        /// RI_RE_WHL
        const PSE_RI_RE_WHL = (1 << 12);
        /// Body Minor
        const PSE_BODY_MINOR = (1 << 13);
        /// Body Major
        const PSE_BODY_MAJOR = (1 << 14);
        /// Setup
        const PSE_SETUP = (1 << 15);
        /// Refuel
        const PSE_REFUEL = (1 << 16);
    }
}

impl_bitflags_from_to_bytes!(PitStopWorkFlags, u32);

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Pit stop at the garage (not a teleport).
///
/// - Reports work done and tyres used.
pub struct Pit {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player that pitted.
    pub plid: PlayerId,

    /// Laps completed.
    pub lapsdone: u16,

    /// Player flags (help settings).
    pub flags: PlayerFlags,

    /// Fuel added during the stop.
    pub fueladd: Fuel,

    /// Penalty state after the stop.
    pub penalty: PenaltyInfo,

    /// Total number of pit stops.
    #[insim(pad_after = 1)]
    pub numstops: u8,

    /// Tyre compounds used (rear-left, rear-right, front-left, front-right).
    pub tyres: [TyreCompound; 4],

    /// Work performed during the stop.
    #[insim(pad_after = 4)]
    pub work: PitStopWorkFlags,
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Pit stop finished event.
pub struct Psf {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player that finished the stop.
    pub plid: PlayerId,

    #[insim(duration= u32, pad_after = 4)]
    /// Total stop time.
    pub stime: Duration,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[non_exhaustive]
/// Pit lane event type used by [Pla].
pub enum PitLaneFact {
    #[default]
    /// Left pitlane
    Exit = 0,

    /// Entered pitlane
    Enter = 1,

    /// Entered for no known reason.
    NoPurpose = 2,

    /// Entered for a drive-through penalty.
    Dt = 3,

    /// Entered for a stop-go penalty.
    Sg = 4,
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Pit lane enter/exit event.
pub struct Pla {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player involved in the pit lane event.
    pub plid: PlayerId,

    /// Pit lane event type.
    #[insim(pad_after = 3)]
    pub fact: PitLaneFact,
}

impl Pla {
    /// Did the player enter the pit lane?
    pub fn entered_pitlane(&self) -> bool {
        self.fact != PitLaneFact::Exit
    }

    /// Did the player exit the pitlane?
    pub fn exited_pitlane(&self) -> bool {
        !self.entered_pitlane()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pit() {
        assert_from_to_bytes!(
            Pit,
            [
                0,   // reqi
                1,   // plid
                25,  // lapsdone (1)
                0,   // lapsdone (2)
                73,  // flags (1)
                6,   // flags (2)
                30,  // fueladd
                0,   // penalty
                1,   // numstops
                0,   // sp3
                255, // tyrerl
                255, // tyrerr
                255, // tyrefl
                255, // tyrefl
                2,   // work (1)
                0,   // work (2)
                0,   // work (3)
                0,   // work (4)
                0,   // spare (1)
                0,   // spare (2)
                0,   // spare (3)
                0,   // spare (4)
            ],
            |parsed: Pit| {
                assert_eq!(parsed.reqi, RequestId(0));
                assert_eq!(parsed.plid, PlayerId(1));
                assert_eq!(parsed.lapsdone, 25);
                assert!(matches!(parsed.fueladd, Fuel::Percentage(30)));
                assert_eq!(parsed.numstops, 1);
            }
        );
    }

    #[test]
    fn test_psf() {
        assert_from_to_bytes!(
            Psf,
            [
                0,  // reqi
                2,  // plid
                89, // stime (1)
                2,  // stime (2)
                3,  // stime (3)
                1,  // stime (4)
                0,  // spare (1)
                0,  // spare (2)
                0,  // spare (3)
                0,  // spare (4)
            ],
            |parsed: Psf| {
                assert_eq!(parsed.reqi, RequestId(0));
                assert_eq!(parsed.plid, PlayerId(2));
                assert_eq!(parsed.stime, Duration::from_millis(16974425));
            }
        );
    }

    #[test]
    fn test_pla() {
        assert_from_to_bytes!(
            Pla,
            [
                0, // reqi
                3, // plid
                4, // fact
                0, // sp1
                0, // sp2
                0, // sp3
            ],
            |parsed: Pla| {
                assert_eq!(parsed.reqi, RequestId(0));
                assert_eq!(parsed.plid, PlayerId(3));
                assert!(matches!(parsed.fact, PitLaneFact::Sg))
            }
        );
    }
}
