use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Used within the [Axm] packet.
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ObjectInfo {
    pub x: i16,
    pub y: i16,

    pub z: u8,
    pub flags: u8,
    pub index: u8,
    pub heading: u8,
}

/// Actionst hat can be taken as part of [Axm].
#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum PmoAction {
    #[default]
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

bitflags::bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    pub struct PmoFlags: u16 {
         const FILE_END = (1 << 0);
         const MOVE_MODIFY = (1 << 1);
         const SELECTION_REAL = (1 << 2);
         const AVOID_CHECK = (1 << 3);
    }
}

/// AutoX Multiple Objects
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Axm {
    pub reqi: RequestId,

    #[bw(calc = info.len() as u8)]
    pub numo: u8,

    pub ucid: ConnectionId,
    pub action: PmoAction,

    #[brw(pad_after = 1)]
    pub flags: PmoFlags,

    #[br(count = numo)]
    pub info: Vec<ObjectInfo>,
}
