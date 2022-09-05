use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::{
    identifiers::{PlayerId, RequestId},
    position::Point,
};

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Camera Position Pack reports the current camera position and state. This packet may also be
/// sent to control the camera.
pub struct Cpp {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub pos: Point<i32>,

    pub h: u16,

    pub p: u16,

    pub r: u16,

    pub viewplid: PlayerId,

    pub ingamecam: u8,

    pub fov: f32,

    pub time: u16,

    pub flags: u16,
}
