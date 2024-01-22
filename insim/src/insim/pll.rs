use insim_core::binrw::{self, binrw};

use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Player Leaves race
pub struct Pll {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,
    /// Unique player id which left
    pub plid: PlayerId,
}
