use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{
    protocol::identifiers::{ConnectionId, RequestId},
    vehicle::Vehicle,
};

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
/// User Selected Car
pub struct Slc {
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    pub cname: Vehicle,
}
