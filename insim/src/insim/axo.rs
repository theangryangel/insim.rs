use insim_core::binrw::{self, binrw};

use crate::identifiers::{PlayerId, RequestId};

/// AutoX Object Contact - if an autocross object is hit (2 second time penalty) this packet is sent
#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Axo {
    /// Non-zero if the packet is a packet request or a reply to a RequestId
    pub reqi: RequestId,

    /// Unique player ID
    pub plid: PlayerId,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_axo() {
        assert_from_to_bytes!(Axo, [1, 2], |axo: Axo| {
            assert_eq!(axo.reqi, RequestId(1));
            assert_eq!(axo.plid, PlayerId(2));
        });
    }
}
