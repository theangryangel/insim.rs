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
/// Insim Multiplayer - LFS sends this when a host is started or joined
pub struct Ism {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[deku(pad_bytes_after = "3")]
    pub host: u8,

    #[deku(bytes = "16")]
    pub hname: CodepageString,
}
