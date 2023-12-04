use insim_core::{
    identifiers::{PlayerId, RequestId},
    binrw::{self, binrw}
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the flag field of [Flg].
#[binrw]
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum FlgType {
    #[default]
    None = 0,

    Blue = 1,

    Yellow = 2,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Race Flag is sent when a flag is waved at a player.
pub struct Flg {
    pub reqi: RequestId,
    pub plid: PlayerId,

    // FIXME this should be a bool
    pub offon: u8,

    pub flag: FlgType,
    #[brw(pad_after = 1)]
    pub carbehind: u8,
}
