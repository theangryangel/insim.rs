use crate::protocol::identifiers::PlayerId;

use super::CarContact;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    type = "u8",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
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

impl Default for Hlvc {
    fn default() -> Self {
        Hlvc::Ground
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Reports incidents that would violate Hot Lap Validity checks.
pub struct Hlv {
    pub reqi: u8,
    pub plid: PlayerId,
    #[deku(pad_bytes_after = "1")]
    pub hlvc: Hlvc,
    pub time: u16,
    pub c: CarContact,
}
