use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{PlayerId, RequestId};

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Split timing
pub struct Spx {
    pub reqi: RequestId,

    pub plid: PlayerId,

    pub stime: u32,

    pub etime: u32,

    pub split: u8,

    pub penalty: u8,

    #[insim(pad_bytes_after = "1")]
    pub numstops: u8,
}
