use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

use super::{PlayerFlags, TyreCompound};
use crate::{
    protocol::identifiers::{PlayerId, RequestId},
};

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct PitStopWorkFlags: u32 {
        NOTHING => 0,
        STOP => (1 << 0),
        FR_DAM => (1 << 1),
        FR_WHL => (1 << 2),
        PSE_LE_FR_DAM => (1 << 3),
        PSE_LE_FR_WHL => (1 << 4),
        PSE_RI_FR_DAM => (1 << 5),
        PSE_RI_FR_WHL => (1 << 6),
        PSE_RE_DAM => (1 << 7),
        PSE_RE_WHL => (1 << 8),
        PSE_LE_RE_DAM => (1 << 9),
        PSE_LE_RE_WHL => (1 << 10),
        PSE_RI_RE_DAM => (1 << 11),
        PSE_RI_RE_WHL => (1 << 12),
        PSE_BODY_MINOR => (1 << 13),
        PSE_BODY_MAJOR => (1 << 14),
        PSE_SETUP => (1 << 15),
        PSE_REFUEL => (1 << 16),
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[insim(
    ctx = "endian: insim::ctx::Endian",
    ctx_default = "insim::ctx::Endian::Little",
    endian = "endian"
)]
/// Pit stop (stop at the garage, not "tele-pit")
pub struct Pit {
    pub reqi: RequestId,

    pub plid: PlayerId,

    pub lapsdone: u16,

    pub flags: PlayerFlags,

    pub fueladd: u8,

    pub penalty: u8,

    #[insim(pad_bytes_after = "1")]
    pub numstops: u8,

    #[insim(count = "4")]
    pub tyres: Vec<TyreCompound>,

    #[insim(bytes = "4", pad_bytes_after = "4")]
    pub work: u32,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[insim(
    ctx = "endian: insim::ctx::Endian",
    ctx_default = "insim::ctx::Endian::Little",
    endian = "endian"
)]
/// Pit Stop Finished
pub struct Psf {
    pub reqi: RequestId,

    pub plid: PlayerId,

    #[insim(pad_bytes_after = "4")]
    pub stime: u32,
}

#[derive(Debug, PartialEq, Eq, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[insim(
    type = "u8",
    ctx = "endian: insim::ctx::Endian",
    ctx_default = "insim::ctx::Endian::Little",
    endian = "endian"
)]
pub enum PitLaneFact {
    #[insim(id = "0")]
    Exit,

    #[insim(id = "1")]
    Enter,

    #[insim(id = "2")]
    EnterNoPurpose,

    #[insim(id = "3")]
    EnterDriveThru,

    #[insim(id = "4")]
    EnterStopGo,
}

impl Default for PitLaneFact {
    fn default() -> Self {
        PitLaneFact::Exit
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// PitLane
pub struct Pla {
    pub reqi: RequestId,

    pub plid: PlayerId,

    #[insim(pad_bytes_after = "3")]
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
