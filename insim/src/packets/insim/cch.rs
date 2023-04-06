use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Default, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum CameraView {
    /// Arcade "follow" view
    #[default]
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

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
// Camera Change
pub struct Cch {
    pub reqi: RequestId,

    pub plid: PlayerId,

    #[insim(pad_bytes_after = "3")]
    pub camera: CameraView,
}
