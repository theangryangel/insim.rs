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
/// Split timing
pub struct Spx {
    pub reqi: RequestId,

    pub plid: PlayerId,

    pub stime: u32,

    pub etime: u32,

    pub split: u8,

    pub penalty: u8,

    #[deku(pad_bytes_after = "1")]
    pub numstops: u8,
}
