use std::time::Duration;

use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
    ser::Limit,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

use super::{PlayerFlags, TyreCompound};

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
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

impl Decodable for PitStopWorkFlags {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Default,
    {
        Ok(Self::from_bits_truncate(u32::decode(buf, limit)?))
    }
}

impl Encodable for PitStopWorkFlags {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<(), insim_core::EncodableError>
    where
        Self: Sized,
    {
        self.bits().encode(buf, limit)?;
        Ok(())
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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

    pub tyres: [TyreCompound; 4],

    #[insim(pad_bytes_after = "4")]
    pub work: u32,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Pit Stop Finished
pub struct Psf {
    pub reqi: RequestId,

    pub plid: PlayerId,

    #[insim(pad_bytes_after = "4")]
    pub stime: Duration,
}

#[derive(Debug, Default, PartialEq, Eq, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum PitLaneFact {
    #[default]
    Exit = 0,

    Enter = 1,

    EnterNoPurpose = 2,

    EnterDriveThru = 3,

    EnterStopGo = 4,
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
