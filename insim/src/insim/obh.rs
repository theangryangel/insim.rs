use insim_core::{
    identifiers::{PlayerId, RequestId},
    binrw::{self, binrw}
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    pub struct ObhFlags: u8 {
        const LAYOUT = (1 << 0);
        const CAN_MOVE = (1 << 1);
        const WAS_MOVING = (1 << 2);
        const ON_SPOT = (1 << 3);
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct CarContact {
    pub direction: u8,
    pub heading: u8,
    pub speed: u8,
    pub z: u8,

    pub x: i16,
    pub y: i16,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Object Hit
pub struct Obh {
    pub reqi: RequestId,
    pub plid: PlayerId,

    pub spclose: u16,
    pub time: u16,

    pub info: CarContact,

    pub x: i16,
    pub y: i16,

    #[brw(pad_after = 1)]
    pub z: u8,
    pub index: u8,
    pub flags: ObhFlags,
}
