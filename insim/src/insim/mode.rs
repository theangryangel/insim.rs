use insim_core::binrw::{self, binrw};

use crate::identifiers::RequestId;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Screen Mode (referred to as originally IS_MOD within Insim.txt)
pub struct Mod {
    #[brw(pad_after = 1)]
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Set to choose 16-bit
    pub bit16: i32,
    /// Refresh rate, zero for default
    pub rr: i32,
    /// Screen width. Zero to switch to windowed mode.
    pub width: i32,
    /// Screen height. Zero to switch to windowed mode.
    pub height: i32,
}
