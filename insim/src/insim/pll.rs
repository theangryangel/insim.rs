use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Player left the race (spectating).
pub struct Pll {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player that left the race.
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
