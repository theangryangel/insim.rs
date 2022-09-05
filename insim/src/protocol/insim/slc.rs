use crate::{
    protocol::identifiers::{ConnectionId, RequestId},
    vehicle::Vehicle,
};
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// User Selected Car
pub struct Slc {
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    pub cname: Vehicle,
}
