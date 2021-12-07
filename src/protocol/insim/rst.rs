use crate::track::Track;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Race Start
pub struct Rst {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub racelaps: u8,

    #[deku(bytes = "1")]
    pub qualmins: u8,

    #[deku(bytes = "1")]
    pub nump: u8,

    #[deku(bytes = "1")]
    pub timing: u8,

    pub track: Track,

    #[deku(bytes = "1")]
    pub weather: u8,

    #[deku(bytes = "1")]
    pub wind: u8,

    #[deku(bytes = "2")]
    pub flags: u16,

    #[deku(bytes = "2")]
    pub numnodes: u16,

    #[deku(bytes = "2")]
    pub finish: u16,

    #[deku(bytes = "2")]
    pub split1: u16,

    #[deku(bytes = "2")]
    pub split2: u16,

    #[deku(bytes = "2")]
    pub split3: u16,
}
