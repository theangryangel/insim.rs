use insim_core::binrw::{self, binrw};

use super::PlayerFlags;
use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Player flags changed
pub struct Pfl {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Player unique Id
    pub plid: PlayerId,

    /// Flags which were altered. See [PlayerFlags].
    #[brw(pad_after = 2)]
    #[read_write_buf(pad_after = 2)]
    pub flags: PlayerFlags,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pfl() {
        assert_from_to_bytes!(
            Pfl,
            [
                0, // reqi
                3, // plid
                9, // flags (1)
                0, // flags (2)
                0, 0,
            ],
            |pfl: Pfl| {
                assert_eq!(pfl.reqi, RequestId(0));
                assert_eq!(pfl.plid, PlayerId(3));
                assert!(pfl
                    .flags
                    .contains(PlayerFlags::AUTOGEARS & PlayerFlags::LEFTSIDE));
            }
        );
    }
}
