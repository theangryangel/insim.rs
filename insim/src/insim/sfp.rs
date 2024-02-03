use insim_core::binrw::{self, binrw};

use super::StaFlags;
use crate::identifiers::RequestId;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// State Flags Pack
pub struct Sfp {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// The state to set/change. See [StaFlags].
    pub flag: StaFlags,

    /// Turn the state on or off
    #[brw(pad_after = 1)]
    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    pub onoff: bool,
}
