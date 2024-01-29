use insim_core::binrw::{self, binrw};

use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Information about a specific vehicle/player. Used within [Nlp].
pub struct NodeLapInfo {
    /// Current path node
    pub node: u16,

    /// Current lap
    pub lap: u16,

    /// Player's unique ID
    pub plid: PlayerId,

    /// Player's race position
    pub position: u8,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Node and Lap packet - similar to Mci without positional information
pub struct Nlp {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    #[bw(calc = info.len() as u8)]
    nump: u8,

    /// Node, lap and position of each player.
    #[br(count = nump)]
    pub info: Vec<NodeLapInfo>,
}
