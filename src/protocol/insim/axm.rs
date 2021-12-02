use deku::prelude::*;
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(endian = "little")]
pub struct ObjectInfo {
    pub x: i16,
    pub y: i16,
    pub z: u8,
    pub flags: u8,
    pub index: u8,
    pub heading: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Serialize, Clone)]
#[deku(type = "u8", endian = "little")]
pub enum PmoAction {
    #[deku(id = "0")]
    LoadingFile,

    #[deku(id = "1")]
    AddObjects,

    #[deku(id = "2")]
    DelObjects,

    #[deku(id = "3")]
    ClearAll,

    #[deku(id = "4")]
    TinyAxm,

    #[deku(id = "5")]
    TtcSel,

    #[deku(id = "6")]
    Selection,

    #[deku(id = "7")]
    Position,

    #[deku(id = "8")]
    GetZ,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// AutoX Multiple Objects
pub struct Axm {
    pub reqi: u8,
    pub numo: u8,
    pub ucid: u8,
    pub action: PmoAction,
    pub flags: u8,

    #[deku(count = "numo")]
    pub info: Vec<ObjectInfo>,
}
