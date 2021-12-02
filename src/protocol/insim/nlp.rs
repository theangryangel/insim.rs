use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(endian = "little")]
pub struct NodeLapInfo {
    #[deku(bytes = "2")]
    pub node: u16,

    #[deku(bytes = "2")]
    pub lap: u16,

    #[deku(bytes = "1")]
    pub plid: u8,

    #[deku(bytes = "1")]
    pub position: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Node and Lap packet - similar to Mci without positional information
pub struct Nlp {
    #[deku(bytes = "1")]
    pub reqi: u8,

    #[deku(bytes = "1")]
    pub nump: u8,

    #[deku(count = "nump")]
    pub nodelap: Vec<NodeLapInfo>,
}
