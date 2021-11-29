use deku::prelude::*;
use serde::Serialize;

use crate::protocol::position::FixedPoint;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Cpp {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    reqi: u8,

    pos: FixedPoint,

    #[deku(bytes = "2")]
    h: u16,

    #[deku(bytes = "2")]
    p: u16,

    #[deku(bytes = "2")]
    r: u16,

    #[deku(bytes = "1")]
    viewplid: u8,

    #[deku(bytes = "1")]
    ingamecam: u8,

    #[deku(bytes = "4")]
    fov: f32,

    #[deku(bytes = "2")]
    time: u16,

    #[deku(bytes = "2")]
    flags: u16,
}
