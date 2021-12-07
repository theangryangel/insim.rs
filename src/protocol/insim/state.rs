use crate::track::Track;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// State
pub struct Sta {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "4")]
    pub replayspeed: f32,

    #[deku(bytes = "2")]
    pub flags: u16,

    #[deku(bytes = "1")]
    pub ingamecam: u8,

    #[deku(bytes = "1")]
    pub viewplid: u8,

    #[deku(bytes = "1")]
    pub nump: u8,

    #[deku(bytes = "1")]
    pub numconns: u8,

    #[deku(bytes = "1")]
    pub numfinished: u8,

    #[deku(bytes = "1")]
    pub raceinprog: u8,

    #[deku(bytes = "1")]
    pub qualmins: u8,

    #[deku(bytes = "1", pad_bytes_after = "2")]
    pub racelaps: u8,

    pub track: Track,

    #[deku(bytes = "1")]
    pub weather: u8,

    #[deku(bytes = "1")]
    pub wind: u8,
}
