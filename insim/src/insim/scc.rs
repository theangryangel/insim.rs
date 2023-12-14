use insim_core::{
    binrw::{self, binrw},
    identifiers::{PlayerId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::CameraView;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Set Car Camera
pub struct Scc {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    pub viewplid: PlayerId,
    #[brw(pad_after = 2)]
    pub ingamecam: CameraView,
}
