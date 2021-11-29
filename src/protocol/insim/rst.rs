use crate::string::InsimString;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct RaceStart {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    racelaps: u8,

    #[deku(bytes = "1")]
    qualmins: u8,

    #[deku(bytes = "1")]
    nump: u8,

    #[deku(bytes = "1")]
    timing: u8,

    #[deku(bytes = "6")]
    track: InsimString,

    #[deku(bytes = "1")]
    weather: u8,

    #[deku(bytes = "1")]
    wind: u8,

    #[deku(bytes = "2")]
    flags: u16,

    #[deku(bytes = "2")]
    numnodes: u16,

    #[deku(bytes = "2")]
    finish: u16,

    #[deku(bytes = "2")]
    split1: u16,

    #[deku(bytes = "2")]
    split2: u16,

    #[deku(bytes = "2")]
    split3: u16,
}
