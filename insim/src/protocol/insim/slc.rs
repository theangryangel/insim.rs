use insim_core::{
    identifiers::{ConnectionId, RequestId},
    prelude::*,
    vehicle::Vehicle,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// User Selected Car
pub struct Slc {
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    pub cname: Vehicle,
}
