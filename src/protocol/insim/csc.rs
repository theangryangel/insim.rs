use super::CarContact;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Serialize, Clone)]
#[deku(type = "u8", endian = "little")]
/// Used within the [Csc] packet to indicate the type of state change.
pub enum CscAction {
    #[deku(id = "0")]
    Stop,

    #[deku(id = "1")]
    Start,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Car State Changed
pub struct Csc {
    pub reqi: u8,

    #[deku(pad_bytes_after = "1")]
    pub plid: u8,

    #[deku(pad_bytes_after = "2")]
    pub action: CscAction,

    pub time: u32,

    pub c: CarContact,
}
