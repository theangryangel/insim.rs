use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
    point::Point,
};

use super::{CameraView, StaFlags};
use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Camera Position Pack reports the current camera position and state. This packet may also be
/// sent to control the camera.
pub struct Cpp {
    #[brw(pad_after = 1)]
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Position vector
    pub pos: Point<i32>,

    /// heading - 0 points along Y axis
    pub h: u16,

    /// Patch
    pub p: u16,

    /// Roll
    pub r: u16,

    /// Unique ID of viewed player (0 = none)
    pub viewplid: PlayerId,

    /// CameraView
    pub ingamecam: CameraView,

    /// Field of View, in degrees
    pub fov: f32,

    /// Time in ms to get there (0 means instant)
    #[br(parse_with = binrw_parse_duration::<u16, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u16, 1, _>)]
    pub time: Duration,

    /// State flags to set
    pub flags: StaFlags,
}

impl_typical_with_request_id!(Cpp);
