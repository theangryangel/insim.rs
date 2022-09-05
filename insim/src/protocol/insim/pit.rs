use super::{PlayerFlags, TyreCompound};
use crate::{
    packet_flags,
    protocol::identifiers::{PlayerId, RequestId},
};
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

packet_flags! {
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

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
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

    #[deku(pad_bytes_after = "1")]
    pub numstops: u8,

    #[deku(count = "4")]
    pub tyres: Vec<TyreCompound>,

    #[deku(bytes = "4", pad_bytes_after = "4")]
    pub work: u32,
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Pit Stop Finished
pub struct Psf {
    pub reqi: RequestId,

    pub plid: PlayerId,

    #[deku(pad_bytes_after = "4")]
    pub stime: u32,
}

#[derive(Debug, PartialEq, Eq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    type = "u8",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub enum PitLaneFact {
    #[deku(id = "0")]
    Exit,

    #[deku(id = "1")]
    Enter,

    #[deku(id = "2")]
    EnterNoPurpose,

    #[deku(id = "3")]
    EnterDriveThru,

    #[deku(id = "4")]
    EnterStopGo,
}

impl Default for PitLaneFact {
    fn default() -> Self {
        PitLaneFact::Exit
    }
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// PitLane
pub struct Pla {
    pub reqi: RequestId,

    pub plid: PlayerId,

    #[deku(pad_bytes_after = "3")]
    pub fact: PitLaneFact,
}

impl Pla {
    pub fn entered(self) -> bool {
        self.fact != PitLaneFact::Exit
    }

    pub fn exited(self) -> bool {
        !self.entered()
    }
}
