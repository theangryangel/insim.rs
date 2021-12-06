use deku::prelude::*;
use serde::Serialize;

/// AutoX Object Contact
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Axo {
    pub reqi: u8,
    pub plid: u8,
}
