use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
// Camera Change
pub struct Cch {
    pub reqi: u8,

    pub plid: u8,

    #[deku(pad_bytes_after = "3")]
    pub camera: u8,
}
