use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{PlayerId, RequestId};

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub struct NodeLapInfo {
    pub node: u16,

    pub lap: u16,

    pub plid: PlayerId,

    pub position: u8,
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Node and Lap packet - similar to Mci without positional information
pub struct Nlp {
    pub reqi: RequestId,

    pub nump: u8,

    #[deku(count = "nump")]
    pub nodelap: Vec<NodeLapInfo>,
}
