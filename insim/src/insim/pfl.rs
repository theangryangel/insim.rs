use insim_core::{
    binrw::{self, binrw},
    identifiers::{PlayerId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::PlayerFlags;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Player Flags
pub struct Pfl {
    pub reqi: RequestId,
    pub plid: PlayerId,

    #[brw(pad_after = 2)]
    pub flags: PlayerFlags,
}
