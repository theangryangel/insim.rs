use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// State Flags Pack
pub struct Sfp {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    pub flag: u16,

    #[deku(pad_bytes_after = "1")]
    pub onoff: u8,
}
