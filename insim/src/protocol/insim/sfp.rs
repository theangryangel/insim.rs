use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::RequestId;

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// State Flags Pack
pub struct Sfp {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub flag: u16,

    #[deku(pad_bytes_after = "1")]
    pub onoff: u8,
}
