use crate::{protocol::identifiers::ConnectionId, vehicle::Vehicle};
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// User Selected Car
pub struct Slc {
    pub reqi: u8,

    pub ucid: ConnectionId,

    pub cname: Vehicle,
}
