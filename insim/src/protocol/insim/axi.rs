use crate::string::CodepageString;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

/// Auto X Info
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Axi {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    pub axstart: u8,
    pub numcp: u8,
    pub numo: u16,

    #[deku(bytes = "32")]
    pub lname: CodepageString,
}
