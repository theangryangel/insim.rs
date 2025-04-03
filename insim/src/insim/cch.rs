use insim_core::{
    binrw::{self, binrw},
    FromToBytes,
};

use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
#[non_exhaustive]
// FIXME: implement From<u8>
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

impl FromToBytes for CameraView {
    fn from_bytes(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let discrim = u8::from_bytes(buf)?;
        let res = match discrim {
            0 => Self::Follow,
            1 => Self::Heli,
            2 => Self::Cam,
            3 => Self::Driver,
            4 => Self::Custom,
            255 => Self::Another,
            found => {
                return Err(insim_core::Error::NoVariantMatch {
                    found: found as u64,
                })
            },
        };

        Ok(res)
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        let discrim = match self {
            CameraView::Follow => 0,
            CameraView::Heli => 1,
            CameraView::Cam => 2,
            CameraView::Driver => 3,
            CameraView::Custom => 4,
            CameraView::Another => 255,
        };

        discrim.to_bytes(buf)?;
        Ok(())
    }
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
