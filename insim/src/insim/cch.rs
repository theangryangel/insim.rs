use insim_core::binrw::{self, binrw};

use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
/// Camera/view identifiers
pub enum CameraView {
    /// Arcade "follow" view
    #[default]
    Follow = 0,

    /// Helicopter view
    Heli = 1,

    /// Static TV camera views
    Cam = 2,

    /// Driver/cockpit view
    Driver = 3,

    /// Custom view
    Custom = 4,

    /// Viewing another player/vehicle
    Another = 255,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Camera Change - sent when an existing driver changes camera
pub struct Cch {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,
    /// Player unique ID
    pub plid: PlayerId,

    #[brw(pad_after = 3)]
    /// View identifier
    pub camera: CameraView,
}
