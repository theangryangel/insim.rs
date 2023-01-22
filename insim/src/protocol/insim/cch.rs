use insim_core::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{PlayerId, RequestId};

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum CameraView {
    /// Arcade "follow" view
    Follow = 0,

    /// Helicopter view
    Helicopter = 1,

    /// Static TV camera views
    TvCamera = 2,

    /// Driver/cockpit view
    Driver = 3,

    /// Custom view
    Custom = 4,

    /// Viewing another player/vehicle
    OtherVehicle = 255,
}

impl Default for CameraView {
    fn default() -> Self {
        CameraView::Follow
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
// Camera Change
pub struct Cch {
    pub reqi: RequestId,

    pub plid: PlayerId,

    #[insim(pad_bytes_after = "3")]
    pub camera: CameraView,
}
