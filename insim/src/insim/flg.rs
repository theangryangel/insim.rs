use insim_core::binrw::{self, binrw};

use crate::identifiers::{PlayerId, RequestId};

/// Enum for the flag field of [Flg].
#[binrw]
#[derive(Default, Debug, Clone, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
#[non_exhaustive]
pub enum FlgType {
    #[default]
    /// Blue flag
    Blue = 1,

    /// Yellow flag
    Yellow = 2,
}

#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
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
    #[read_write_buf(pad_after = 1)]
    pub carbehind: PlayerId,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_flg() {
        assert_from_to_bytes!(
            Flg,
            [
                0,  // reqi
                3,  // plid
                1,  // offon
                2,  // flag
                14, // carbehind
                0,  // sp3
            ],
            |flg: Flg| {
                assert_eq!(flg.offon, true);
                assert!(matches!(flg.flag, FlgType::Yellow));
            }
        );
    }
}
