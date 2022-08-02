use crate::protocol::identifiers::ConnectionId;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

/// Used within the [Axm] packet.
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub struct ObjectInfo {
    pub x: i16,
    pub y: i16,
    pub z: u8,
    pub flags: u8,
    pub index: u8,
    pub heading: u8,
}

/// Actions that can be taken as part of [Axm].
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    type = "u8",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
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

impl Default for PmoAction {
    fn default() -> Self {
        PmoAction::LoadingFile
    }
}

/// AutoX Multiple Objects
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub struct Axm {
    pub reqi: u8,
    pub numo: u8,
    pub ucid: ConnectionId,
    pub action: PmoAction,
    pub flags: u8,

    #[deku(count = "numo")]
    pub info: Vec<ObjectInfo>,
}
