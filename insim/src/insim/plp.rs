use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Player tele-pitted (Shift+P).
pub struct Plp {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player that tele-pitted.
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
