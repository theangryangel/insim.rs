use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::position::FixedPoint;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Camera Position Pack reports the current camera position and state. This packet may also be
/// sent to control the camera.
pub struct Cpp {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    pub pos: FixedPoint,

    pub h: u16,

    pub p: u16,

    pub r: u16,

    pub viewplid: u8,

    pub ingamecam: u8,

    pub fov: f32,

    pub time: u16,

    pub flags: u16,
}
