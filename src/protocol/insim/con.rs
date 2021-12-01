use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(endian = "little")]
pub struct ContactInfo {
    #[deku(bytes = "1")]
    plid: u8,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    info: u8,

    #[deku(bytes = "1")]
    steer: u8,

    #[deku(bytes = "1")]
    thrbrk: u8,

    #[deku(bytes = "1")]
    cluhan: u8,

    #[deku(bytes = "1")]
    gearsp: u8,

    #[deku(bytes = "1")]
    speed: u8,

    #[deku(bytes = "1")]
    direction: u8,

    #[deku(bytes = "1")]
    heading: u8,

    #[deku(bytes = "1")]
    accelf: u8,

    #[deku(bytes = "1")]
    acelr: u8,

    #[deku(bytes = "2")]
    x: i16,

    #[deku(bytes = "2")]
    y: i16,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Contact {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    reqi: u8,

    #[deku(bytes = "2")]
    spclose: u16,

    #[deku(bytes = "2")]
    time: u16,

    a: ContactInfo,
    b: ContactInfo,
}
