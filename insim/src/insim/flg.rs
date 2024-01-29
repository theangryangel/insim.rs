use insim_core::binrw::{self, binrw};

use crate::identifiers::{PlayerId, RequestId};

/// Enum for the flag field of [Flg].
#[binrw]
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum FlgType {
    #[default]
    /// Blue flag
    Blue = 1,

    /// Yellow flag
    Yellow = 2,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Race Flag is sent when a flag is waved at a player.
pub struct Flg {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique player ID
    pub plid: PlayerId,

    #[br(map = |x: u8| x != 0)]
    #[bw(map = |&x| x as u8)]
    /// Flag on/off
    pub offon: bool,

    /// What type of flag is being waved
    pub flag: FlgType,

    /// Player behind
    #[brw(pad_after = 1)]
    pub carbehind: PlayerId,
}
