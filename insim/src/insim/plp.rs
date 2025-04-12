use insim_core::binrw::{self, binrw};

use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default, insim_macros::FromToBytes)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Player Tele-pits (shift+P in game)
pub struct Plp {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique player ID which tele-pitted
    pub plid: PlayerId,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_plp() {
        assert_from_to_bytes!(Plp, [1, 9], |parsed: Plp| {
            assert_eq!(parsed.reqi, RequestId(1));
            assert_eq!(parsed.plid, PlayerId(9));
        })
    }
}
