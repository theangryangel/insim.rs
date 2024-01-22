use insim_core::binrw::{self, binrw};

use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Car Reset packet indicates a vehicle has been reset or that a vehicle should be reset by the
/// server.
pub struct Crs {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,
    /// Unique player ID that was reset, or should be reset
    pub plid: PlayerId,
}
