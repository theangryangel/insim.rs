use insim_core::{
    identifiers::{PlayerId, RequestId},
    point::Point,
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Camera Position Pack reports the current camera position and state. This packet may also be
/// sent to control the camera.
pub struct Cpp {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub pos: Point<i32>,

    pub h: u16,

    pub p: u16,

    pub r: u16,

    pub viewplid: PlayerId,

    pub ingamecam: u8,

    pub fov: f32,

    // should this be a special duration? do we need a serde-like 'with' annotation?
    pub time: u16,

    pub flags: u16,
}
