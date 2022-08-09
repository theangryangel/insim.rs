use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::PlayerId;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Car Reset packet indicates a vehicle has been reset or that a vehicle should be reset by the
/// server.
pub struct Crs {
    pub reqi: u8,

    pub plid: PlayerId,
}
