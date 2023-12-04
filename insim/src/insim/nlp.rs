use insim_core::{
    binrw::{self, binrw},
    identifiers::{PlayerId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct NodeLapInfo {
    pub node: u16,
    pub lap: u16,
    pub plid: PlayerId,
    pub position: u8,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Node and Lap packet - similar to Mci without positional information
pub struct Nlp {
    pub reqi: RequestId,

    #[bw(calc = nodelap.len() as u8)]
    nump: u8,

    #[br(count = nump)]
    pub nodelap: Vec<NodeLapInfo>,
}
