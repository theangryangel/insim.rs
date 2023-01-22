use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{PlayerId, RequestId};

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Set Car Camera
pub struct Scc {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub viewplid: PlayerId,

    #[deku(pad_bytes_after = "2")]
    pub ingamecam: u8,
}
