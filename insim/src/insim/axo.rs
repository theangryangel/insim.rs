use crate::identifiers::{PlayerId, RequestId};

/// AutoX object contact event.
///
/// - Sent when an autocross object is hit.
#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Axo {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player that hit the object.
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
