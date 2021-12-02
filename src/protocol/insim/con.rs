use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(endian = "little")]
pub struct ConInfo {
    #[deku(bytes = "1")]
    pub plid: u8,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub info: u8,

    #[deku(bytes = "1")]
    pub steer: u8,

    #[deku(bytes = "1")]
    pub thrbrk: u8,

    #[deku(bytes = "1")]
    pub cluhan: u8,

    #[deku(bytes = "1")]
    pub gearsp: u8,

    #[deku(bytes = "1")]
    pub speed: u8,

    #[deku(bytes = "1")]
    pub direction: u8,

    #[deku(bytes = "1")]
    pub heading: u8,

    #[deku(bytes = "1")]
    pub accelf: u8,

    #[deku(bytes = "1")]
    pub acelr: u8,

    #[deku(bytes = "2")]
    pub x: i16,

    #[deku(bytes = "2")]
    pub y: i16,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Contact
pub struct Con {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "2")]
    pub spclose: u16,

    #[deku(bytes = "2")]
    pub time: u16,

    pub a: ConInfo,
    pub b: ConInfo,
}
