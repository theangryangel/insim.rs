use crate::{protocol::identifiers::RequestId, string::CodepageString};
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Message Type - Send a message to LFS as if typed by a user
pub struct Mst {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[deku(bytes = "64")]
    pub msg: CodepageString,
}
