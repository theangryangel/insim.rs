use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Tiny {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub subtype: u8,
}
