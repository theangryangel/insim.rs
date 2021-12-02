use deku::prelude::*;
use serde::Serialize;

use crate::protocol::position::FixedPoint;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Camera Position Pack
pub struct Cpp {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    pub pos: FixedPoint,

    #[deku(bytes = "2")]
    pub h: u16,

    #[deku(bytes = "2")]
    pub p: u16,

    #[deku(bytes = "2")]
    pub r: u16,

    #[deku(bytes = "1")]
    pub viewplid: u8,

    #[deku(bytes = "1")]
    pub ingamecam: u8,

    #[deku(bytes = "4")]
    pub fov: f32,

    #[deku(bytes = "2")]
    pub time: u16,

    #[deku(bytes = "2")]
    pub flags: u16,
}
