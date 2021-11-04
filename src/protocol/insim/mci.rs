use crate::into_packet_variant;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(endian = "little")]
pub struct CompCar {
    #[deku(bytes = "2")]
    node: u16,

    #[deku(bytes = "2")]
    lap: u16,

    #[deku(bytes = "1")]
    plid: u8,

    #[deku(bytes = "1")]
    position: u8,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    info: u8,

    // sp3 handled by pad_bytes_after
    #[deku(bytes = "4")]
    x: i32, // X map (65536 = 1 metre)

    #[deku(bytes = "4")]
    y: i32, // Y map (65536 = 1 metre)

    #[deku(bytes = "4")]
    z: i32, // Z alt (65536 = 1 metre)

    #[deku(bytes = "2")]
    speed: u16, // speed (32768 = 100 m/s)

    #[deku(bytes = "2")]
    direction: u16, // direction of car's motion : 0 = world y direction, 32768 = 180 deg

    #[deku(bytes = "2")]
    heading: u16, // direction of forward axis : 0 = world y direction, 32768 = 180 deg

    #[deku(bytes = "2")]
    angvel: i16, // signed, rate of change of heading : (16384 = 360 deg/s)
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct MultiCarInfo {
    #[deku(bytes = "1")]
    reqi: u8,
    #[deku(bytes = "1")]
    numc: u8,

    #[deku(count = "numc")]
    info: Vec<CompCar>,
}

into_packet_variant!(MultiCarInfo, MultiCarInfo);
