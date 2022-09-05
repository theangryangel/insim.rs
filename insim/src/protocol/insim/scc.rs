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
/// Set Car Camera
pub struct Scc {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub viewplid: PlayerId,

    #[deku(pad_bytes_after = "2")]
    pub ingamecam: u8,
}
