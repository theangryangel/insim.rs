use crate::{protocol::identifiers::RequestId, string::istring};
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
/// Version
pub struct Version {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[deku(
        reader = "istring::read(deku::rest, 8)",
        writer = "istring::write(deku::output, &self.version, 8)"
    )]
    pub version: String,

    #[deku(
        reader = "istring::read(deku::rest, 6)",
        writer = "istring::write(deku::output, &self.product, 6)"
    )]
    pub product: String,

    #[deku(pad_bytes_after = "1")]
    pub insimver: u8,
}
