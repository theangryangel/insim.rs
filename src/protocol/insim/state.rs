use crate::into_packet_variant;
use crate::string::InsimString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Sta {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    reqi: u8,

    #[deku(bytes = "4")]
    replayspeed: f32,

    #[deku(bytes = "2")]
    flags: u16,

    #[deku(bytes = "1")]
    ingamecam: u8,

    #[deku(bytes = "1")]
    viewplid: u8,

    #[deku(bytes = "1")]
    nump: u8,

    #[deku(bytes = "1")]
    numconns: u8,

    #[deku(bytes = "1")]
    numfinished: u8,

    #[deku(bytes = "1")]
    raceinprog: u8,

    #[deku(bytes = "1")]
    qualmins: u8,

    #[deku(bytes = "1", pad_bytes_after = "2")]
    racelaps: u8,

    #[deku(bytes = "6")]
    track: InsimString,

    #[deku(bytes = "1")]
    weather: u8,

    #[deku(bytes = "1")]
    wind: u8,
}

into_packet_variant!(Sta, State);
