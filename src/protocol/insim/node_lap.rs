use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(endian = "little")]
pub struct NodeLapInfo {
    #[deku(bytes = "2")]
    node: u16,

    #[deku(bytes = "2")]
    lap: u16,

    #[deku(bytes = "1")]
    plid: u8,

    #[deku(bytes = "1")]
    position: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct NodeLap {
    #[deku(bytes = "1")]
    reqi: u8,

    #[deku(bytes = "1")]
    nump: u8,

    #[deku(count = "nump")]
    nodelap: Vec<NodeLapInfo>,
}
