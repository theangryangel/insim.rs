use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::CameraView;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Set Car Camera
pub struct Scc {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub viewplid: PlayerId,
    #[insim(pad_bytes_after = "2")]
    pub ingamecam: CameraView,
}
