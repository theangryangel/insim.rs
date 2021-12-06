use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(endian = "little")]
pub struct NodeLapInfo {
    pub node: u16,

    pub lap: u16,

    pub plid: u8,

    pub position: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Node and Lap packet - similar to Mci without positional information
pub struct Nlp {
    pub reqi: u8,

    pub nump: u8,

    #[deku(count = "nump")]
    pub nodelap: Vec<NodeLapInfo>,
}
