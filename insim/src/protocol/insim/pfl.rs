use crate::protocol::identifiers::{PlayerId, RequestId};

use super::PlayerFlags;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Player Flags
pub struct Pfl {
    pub reqi: RequestId,

    pub plid: PlayerId,

    #[deku(pad_bytes_after = "2")]
    pub flags: PlayerFlags,
}
