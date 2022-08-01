use crate::{protocol::identifiers::ConnectionId, vehicle::Vehicle};
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// User Selected Car
pub struct Slc {
    pub reqi: u8,

    pub ucid: ConnectionId,

    pub cname: Vehicle,
}
