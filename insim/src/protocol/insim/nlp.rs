use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{PlayerId, RequestId};

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct NodeLapInfo {
    pub node: u16,

    pub lap: u16,

    pub plid: PlayerId,

    pub position: u8,
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Node and Lap packet - similar to Mci without positional information
pub struct Nlp {
    pub reqi: RequestId,

    pub nump: u8,

    #[deku(count = "nump")]
    pub nodelap: Vec<NodeLapInfo>,
}
