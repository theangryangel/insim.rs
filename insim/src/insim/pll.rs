use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Player Leaves race
pub struct Pll {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique player id which left
    pub plid: PlayerId,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pll() {
        assert_from_to_bytes!(Pll, [0, 12], |parsed: Pll| {
            assert_eq!(parsed.reqi, RequestId(0));
            assert_eq!(parsed.plid, PlayerId(12));
        });
    }
}
