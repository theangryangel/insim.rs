use insim_core::{
    binrw::{self, binrw},
    identifiers::RequestId,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum OcoAction {
    #[default]
    LightsReset = 4,

    LightsSet = 5,

    LightsUnset = 6,
}

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum OcoIndex {
    AxoStartLights = 149,

    #[default]
    MainLights = 240,
}

bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    pub struct OcoLights: u8 {
        const RED1 = (1 << 0);
        const RED2 = (1 << 1);
        const RED3 = (1 << 2);
        const GREEN = (1 << 3);
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Object Control
/// Used to switch start lights
pub struct Oco {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    pub action: OcoAction,
    pub index: OcoIndex,
    pub identifer: u8,
    pub lights: OcoLights,
}
