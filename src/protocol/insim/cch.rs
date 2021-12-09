use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(type = "u8", endian = "little")]
pub enum CameraView {
    #[deku(id = "0")]
    /// Arcade "follow" view
    Follow,

    #[deku(id = "1")]
    /// Helicopter view
    Helicopter,

    #[deku(id = "2")]
    /// Static TV camera views
    TvCamera,

    #[deku(id = "3")]
    /// Driver/cockpit view
    Driver,

    #[deku(id = "4")]
    /// Custom view
    Custom,

    #[deku(id = "255")]
    /// Viewing another player/vehicle
    OtherVehicle,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
// Camera Change
pub struct Cch {
    pub reqi: u8,

    pub plid: u8,

    #[deku(pad_bytes_after = "3")]
    pub camera: CameraView,
}
