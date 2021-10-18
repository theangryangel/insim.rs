use crate::into_packet_variant;
use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Tiny {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub subtype: u8,
}

into_packet_variant!(Tiny, Tiny);
