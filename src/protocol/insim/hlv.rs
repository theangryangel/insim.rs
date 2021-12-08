use super::CarContact;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(type = "u8", endian = "little")]
/// Used within [Hlv] to indicate the hotlap validity failure reason.
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

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Reports incidents that would violate Hot Lap Validity checks.
pub struct Hlv {
    pub reqi: u8,
    pub plid: u8,
    #[deku(pad_bytes_after = "1")]
    pub hlvc: Hlvc,
    pub time: u16,
    pub c: CarContact,
}
