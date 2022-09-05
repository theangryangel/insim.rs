use crate::{protocol::identifiers::RequestId, string::CodepageString};
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

/// Auto X Info
#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub struct Axi {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub axstart: u8,
    pub numcp: u8,
    pub numo: u16,

    #[deku(bytes = "32")]
    pub lname: CodepageString,
}
