use crate::conversion;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(endian = "little")]
pub struct CompCar {
    pub node: u16,
    pub lap: u16,
    pub plid: u8,
    pub position: u8,

    #[deku(pad_bytes_after = "1")]
    pub info: u8, // FIXME: implement flags
    pub x: i32,         // X map (65536 = 1 metre)
    pub y: i32,         // Y map (65536 = 1 metre)
    pub z: i32,         // Z alt (65536 = 1 metre)
    pub speed: u16,     // speed (32768 = 100 m/s)
    pub direction: u16, // direction of car's motion : 0 = world y direction, 32768 = 180 deg
    pub heading: u16,   // direction of forward axis : 0 = world y direction, 32768 = 180 deg
    pub angvel: i16,    // signed, rate of change of heading : (16384 = 360 deg/s)
}

impl CompCar {
    pub fn mph(&self) -> f32 {
        conversion::speed::to_mph(self.speed)
    }
    pub fn kmph(&self) -> f32 {
        conversion::speed::to_kmph(self.speed)
    }
    pub fn mps(&self) -> f32 {
        conversion::speed::to_mps(self.speed)
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Multi Car Info - positional information for upto 8 vehicles
pub struct Mci {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub numc: u8,

    #[deku(count = "numc")]
    pub info: Vec<CompCar>,
}
