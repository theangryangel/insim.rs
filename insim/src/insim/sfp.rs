use insim_core::{identifiers::RequestId, binrw::{self, binrw}};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::StaFlags;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// State Flags Pack
pub struct Sfp {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    pub flag: StaFlags,

    #[brw(pad_after = 1)]
    pub onoff: u8,
}
