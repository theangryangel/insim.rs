use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{PlayerId, RequestId};

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Reorder
pub struct Reo {
    pub reqi: RequestId,

    pub nump: u8,

    #[deku(count = "40")]
    pub plid: Vec<PlayerId>,
}
