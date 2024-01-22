use insim_core::binrw::{self, binrw};

use crate::identifiers::{PlayerId, RequestId};

use super::CameraView;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Set Car Camera
pub struct Scc {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// Player ID
    pub viewplid: PlayerId,

    /// How to manipulate the camera. See [CameraView].
    #[brw(pad_after = 2)]
    pub ingamecam: CameraView,
}
