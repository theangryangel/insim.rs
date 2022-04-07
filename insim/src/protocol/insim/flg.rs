use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the flag field of [Flg].
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(type = "u8", endian = "little")]
pub enum FlgType {
    #[deku(id = "0")]
    None,

    #[deku(id = "1")]
    Blue,

    #[deku(id = "2")]
    Yellow,
}

impl Default for FlgType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Race Flag is sent when a flag is waved at a player.
pub struct Flg {
    pub reqi: u8,

    pub plid: u8,

    pub offon: u8,

    pub flag: FlgType,

    #[deku(pad_bytes_after = "1")]
    pub carbehind: u8,
}
