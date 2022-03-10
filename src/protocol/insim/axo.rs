use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

/// AutoX Object Contact
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Axo {
    pub reqi: u8,
    pub plid: u8,
}
