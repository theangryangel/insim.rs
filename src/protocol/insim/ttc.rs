use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Serialize, Clone)]
#[deku(type = "u8", endian = "little")]
pub enum TtcType {
    #[deku(id = "0")]
    None,

    #[deku(id = "1")]
    Selection, // Send Axm for the current layout editor selection

    #[deku(id = "2")]
    SelectionStart, // Send Axm every time the selection changes

    #[deku(id = "3")]
    SelectionStop, // Stop sending Axm's
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// General purpose Target To Connection packet
pub struct Ttc {
    pub reqi: u8,

    pub subtype: TtcType,

    pub ucid: u8,

    pub b1: u8,
    pub b2: u8,
    pub b3: u8,
}
