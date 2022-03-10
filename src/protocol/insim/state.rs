use super::Wind;
use crate::track::Track;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// State
pub struct Sta {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    pub replayspeed: f32,

    pub flags: u16,

    pub ingamecam: u8,

    pub viewplid: u8,

    pub nump: u8,

    pub numconns: u8,

    pub numfinished: u8,

    pub raceinprog: u8,

    pub qualmins: u8,

    #[deku(pad_bytes_after = "2")]
    pub racelaps: u8,

    pub track: Track,

    pub weather: u8,

    pub wind: Wind,
}
