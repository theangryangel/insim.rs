use crate::protocol::identifiers::PlayerId;

use super::PlayerFlags;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Player Flags
pub struct Pfl {
    pub reqi: u8,

    pub plid: PlayerId,

    #[deku(pad_bytes_after = "2")]
    pub flags: PlayerFlags,
}
