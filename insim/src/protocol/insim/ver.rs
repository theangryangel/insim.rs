use crate::string::istring;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Version
pub struct Version {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

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

    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub insimver: u8,
}