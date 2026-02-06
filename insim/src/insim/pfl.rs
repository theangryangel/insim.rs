use super::PlayerFlags;
use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Player flags changed.
///
/// - Reports changes to assist and input settings.
pub struct Pfl {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player whose flags changed.
    pub plid: PlayerId,

    /// Updated player flags.
    #[insim(pad_after = 2)]
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
                assert!(
                    pfl.flags
                        .contains(PlayerFlags::AUTOGEARS & PlayerFlags::LEFTSIDE)
                );
            }
        );
    }
}
