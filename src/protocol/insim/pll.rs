use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Player Leaves race
pub struct Pll {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub plid: u8,
}
