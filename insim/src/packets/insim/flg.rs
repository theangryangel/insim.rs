use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the flag field of [Flg].
#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum FlgType {
    None = 0,

    Blue = 1,

    Yellow = 2,
}

impl Default for FlgType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Race Flag is sent when a flag is waved at a player.
pub struct Flg {
    pub reqi: RequestId,

    pub plid: PlayerId,

    pub offon: u8,

    pub flag: FlgType,

    #[insim(pad_bytes_after = "1")]
    pub carbehind: u8,
}
