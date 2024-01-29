use insim_core::binrw::{self, binrw};

use crate::identifiers::{PlayerId, RequestId};

use super::PlayerFlags;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Player flags changed
pub struct Pfl {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Player unique Id
    pub plid: PlayerId,

    /// Flags which were altered. See [PlayerFlags].
    #[brw(pad_after = 2)]
    pub flags: PlayerFlags,
}
