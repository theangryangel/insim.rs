use insim_core::{
    identifiers::{ConnectionId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Used within the [Axm] packet.
#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ObjectInfo {
    pub x: i16,
    pub y: i16,
    pub z: u8,
    pub flags: u8,
    pub index: u8,
    pub heading: u8,
}

/// Actions that can be taken as part of [Axm].
#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum PmoAction {
    LoadingFile = 0,

    AddObjects = 1,

    DelObjects = 2,

    ClearAll = 3,

    TinyAxm = 4,

    TtcSel = 5,

    Selection = 6,

    Position = 7,

    GetZ = 8,
}

impl Default for PmoAction {
    fn default() -> Self {
        PmoAction::LoadingFile
    }
}

/// AutoX Multiple Objects
#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Axm {
    pub reqi: RequestId,
    pub numo: u8,
    pub ucid: ConnectionId,
    pub action: PmoAction,
    pub flags: u8,

    #[insim(count = "numo")]
    pub info: Vec<ObjectInfo>,
}
