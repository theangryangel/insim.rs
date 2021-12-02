use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// AutoX Object Contact
pub struct Axo {
    pub reqi: u8,
    pub plid: u8,
}
