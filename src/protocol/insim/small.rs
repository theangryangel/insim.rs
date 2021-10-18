use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Small {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub subtype: u8,

    #[deku(bytes = "4")]
    pub uval: u32,
}
