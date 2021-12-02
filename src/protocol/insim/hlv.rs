use super::CarContact;
use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Serialize, Clone)]
#[deku(type = "u8", endian = "little")]
pub enum Hlvc {
    #[deku(id = "0")]
    Ground,

    #[deku(id = "1")]
    Wall,

    #[deku(id = "4")]
    Speeding,

    #[deku(id = "5")]
    OutOfBounds,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Hot Lap Validity
pub struct Hlv {
    pub reqi: u8,
    pub plid: u8,
    #[deku(pad_bytes_after = "1")]
    pub hlvc: Hlvc,
    pub time: u16,
    pub c: CarContact,
}
