use crate::string::IString;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Version
pub struct Version {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(bytes = "8")]
    pub version: IString,

    #[deku(bytes = "6")]
    pub product: IString,

    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub insimver: u8,
}
