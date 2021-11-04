use crate::into_packet_variant;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Small {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub subtype: u8,

    #[deku(bytes = "4")]
    pub uval: u32,
}

into_packet_variant!(Small, Small);
