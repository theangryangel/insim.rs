use insim_core::{
    binrw::{self, binrw},
    identifiers::{PlayerId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Used within [Con] packet to give a break down of information about the Contact between the two
/// players.
pub struct ConInfo {
    pub plid: PlayerId,
    #[brw(pad_after = 1)]
    pub info: u8,
    pub steer: u8,

    pub thrbrk: u8,
    pub cluhan: u8,
    pub gearsp: u8,
    pub speed: u8,

    pub direction: u8,
    pub heading: u8,
    pub accelf: u8,
    pub acelr: u8,

    pub x: i16,
    pub y: i16,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Contact
pub struct Con {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    // TODO: abstract spclose and time
    pub spclose: u16,
    pub time: u16,

    pub a: ConInfo,
    pub b: ConInfo,
}
