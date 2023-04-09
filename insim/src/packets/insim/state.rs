use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
    track::Track,
    wind::Wind,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// State
pub struct Sta {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub replayspeed: f32,

    pub flags: u16,
    pub ingamecam: u8,
    pub viewplid: PlayerId,

    pub nump: u8,
    pub numconns: u8,
    pub numfinished: u8,
    pub raceinprog: u8,

    pub qualmins: u8,
    #[insim(pad_bytes_after = "1")]
    pub racelaps: u8,
    pub serverstatus: u8,

    pub track: Track,
    pub weather: u8,
    pub wind: Wind,
}
