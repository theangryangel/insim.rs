use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Car reset event or reset request.
///
/// - Sent when a car is reset, or used to request a reset.
pub struct Crs {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,
    /// Player that was reset (or should be reset).
    pub plid: PlayerId,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_crs() {
        assert_from_to_bytes!(Crs, [1, 9], |crs: Crs| {
            assert_eq!(crs.reqi, RequestId(1));
            assert_eq!(crs.plid, PlayerId(9));
        });
    }
}
