use insim_core::{
    binrw::{self, binrw},
    vehicle::Vehicle,
};

use crate::identifiers::{ConnectionId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// User Selected Car - sent when a connection selects a car (empty if no car)
pub struct Slc {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection ID which selected a vehicle
    pub ucid: ConnectionId,

    /// Vehicle which the connection selected
    pub cname: Vehicle,
}
