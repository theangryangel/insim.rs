use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(endian = "little")]
/// Used within [Con] packet to give a break down of information about the Contact between the two
/// players.
pub struct ConInfo {
    pub plid: u8,

    #[deku(pad_bytes_after = "1")]
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

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Contact
pub struct Con {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    pub spclose: u16,

    pub time: u16,

    pub a: ConInfo,
    pub b: ConInfo,
}
