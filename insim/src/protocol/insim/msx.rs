use crate::string::CodepageString;
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
/// Extended Message (like [Mst](super::Mst), but longer)
pub struct Msx {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "96")]
    pub msg: CodepageString,
}
