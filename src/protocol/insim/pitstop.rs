use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct PitStopStart {
    #[deku(bytes = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    plid: u8,

    #[deku(bytes = "2")]
    lapsdone: u16,

    #[deku(bytes = "2")]
    flags: u16,

    #[deku(bytes = "1")]
    fueladd: u8,

    #[deku(bytes = "1")]
    penalty: u8,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    numstops: u8,

    #[deku(count = "4", bytes = "4")]
    tyres: Vec<u8>,

    #[deku(bytes = "4", pad_bytes_after = "1")]
    work: u32,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct PitStopFinish {
    #[deku(bytes = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    plid: u8,

    #[deku(bytes = "4", pad_bytes_after = "4")]
    stime: u32,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct PitLane {
    #[deku(bytes = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    plid: u8,

    #[deku(bytes = "1", pad_bytes_after = "3")]
    fact: u8,
}
