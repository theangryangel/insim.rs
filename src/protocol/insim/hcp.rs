use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
/// Car Handicaps
pub struct HcpCarHandicap {
    pub added_mass: u8,
    pub intake_restriction: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Handicaps
pub struct Hcp {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    #[deku(count = "32")]
    pub info: Vec<HcpCarHandicap>,
}
