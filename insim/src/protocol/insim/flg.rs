use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::PlayerId;

/// Enum for the flag field of [Flg].
#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    type = "u8",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
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
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Race Flag is sent when a flag is waved at a player.
pub struct Flg {
    pub reqi: u8,

    pub plid: PlayerId,

    pub offon: u8,

    pub flag: FlgType,

    #[deku(pad_bytes_after = "1")]
    pub carbehind: u8,
}
