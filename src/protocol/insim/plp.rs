use crate::into_packet_variant;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Plp {
    #[deku(bytes = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    plid: u8,
}

into_packet_variant!(Plp, Plp);
