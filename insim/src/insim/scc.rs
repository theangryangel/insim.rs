use super::CameraView;
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Set the in-game camera for a player.
///
/// - Updates the camera view for the target player.
pub struct Scc {
    /// Request identifier echoed by replies.
    #[insim(pad_after = 1)]
    pub reqi: RequestId,

    /// Player whose camera should change.
    pub viewplid: PlayerId,

    /// Camera view to apply.
    #[insim(pad_after = 2)]
    pub ingamecam: CameraView,
}

impl_typical_with_request_id!(Scc);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scc() {
        assert_from_to_bytes!(
            Scc,
            [
                1, // reqi
                0, 1, // viewplid
                3, // ingamecam
                0, 0,
            ],
            |parsed: Scc| {
                assert_eq!(parsed.reqi, RequestId(1));
            }
        );
    }
}
