use std::time::Duration;

use glam::IVec3;

use super::{CameraView, StaFlags};
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
/// Camera Position Pack reports the current camera position and state. This packet may also be
/// sent to control the camera.
pub struct Cpp {
    #[insim(pad_after = 1)]
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Position vector
    pub pos: IVec3,

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
    #[insim(duration(milliseconds = u16))]
    pub time: Duration,

    /// State flags to set
    pub flags: StaFlags,
}

impl_typical_with_request_id!(Cpp);

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_cpp() {
        assert_from_to_bytes!(
            Cpp,
            [
                1,   // reqi
                0,   // zero
                1,   // x (1)
                0,   // x (2)
                0,   // x (3)
                0,   // x (4)
                255, // y (1)
                255, // y (2)
                255, // y (3)
                127, // y (4)
                0,   // z (1)
                0,   // z (2)
                0,   // z (3)
                128, // z (4)
                255, // h (1)
                255, // h (2)
                200, // p (1)
                1,   // p (2)
                39,  // r (1)
                0,   // r (0)
                32,  // viewplid
                4,   // ingamecam
                0,   // fov (1)
                0,   // fov (2)
                32,  // fov (3)
                66,  // fov (4)
                200, // time (1)
                0,   // time (2)
                0,   // flags (1)
                32,  // flags (2)
            ],
            |parsed: Cpp| {
                assert_eq!(parsed.reqi, RequestId(1));
                assert_eq!(parsed.time, Duration::from_millis(200));
            }
        )
    }
}
