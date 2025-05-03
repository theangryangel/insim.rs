use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[non_exhaustive]
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

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Camera Change - sent when an existing driver changes camera
pub struct Cch {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,
    /// Player unique ID
    pub plid: PlayerId,

    #[read_write_buf(pad_after = 3)]
    /// View identifier
    pub camera: CameraView,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cch() {
        assert_from_to_bytes!(
            Cch,
            [
                0, // reqi
                3, // plid
                4, // camera
                0, 0, 0,
            ],
            |parsed: Cch| {
                assert_eq!(parsed.reqi, RequestId(0));
                assert_eq!(parsed.plid, PlayerId(3));
                assert!(matches!(parsed.camera, CameraView::Custom));
            }
        );
    }

    #[test]
    fn test_camera_view() {
        assert_from_to_bytes!(CameraView, [1], |parsed: CameraView| {
            assert!(matches!(parsed, CameraView::Heli));
        });

        assert_from_to_bytes!(CameraView, [255], |parsed: CameraView| {
            assert!(matches!(parsed, CameraView::Another));
        });
    }
}
