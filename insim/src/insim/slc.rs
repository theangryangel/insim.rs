use insim_core::{
    identifiers::{ConnectionId, RequestId},
    vehicle::Vehicle,
    binrw::{self, binrw}
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// User Selected Car
pub struct Slc {
    pub reqi: RequestId,
    pub ucid: ConnectionId,

    pub cname: Vehicle,
}
