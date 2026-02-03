use crate::identifiers::{PlayerId, RequestId};

/// Flag type reported by [Flg].
#[derive(Default, Debug, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[non_exhaustive]
pub enum FlgType {
    #[default]
    /// Blue flag
    Blue = 1,

    /// Yellow flag
    Yellow = 2,
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Flag state change for a player.
///
/// - Reports when blue or yellow flags are applied or cleared.
pub struct Flg {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player receiving the flag.
    pub plid: PlayerId,

    /// Flag on/off state.
    pub offon: bool,

    /// Flag type.
    pub flag: FlgType,

    /// Player behind (for blue flags).
    #[insim(pad_after = 1)]
    pub carbehind: PlayerId,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_flg() {
        assert_from_to_bytes!(
            Flg,
            [
                0,  // reqi
                3,  // plid
                1,  // offon
                2,  // flag
                14, // carbehind
                0,  // sp3
            ],
            |flg: Flg| {
                assert_eq!(flg.offon, true);
                assert!(matches!(flg.flag, FlgType::Yellow));
            }
        );
    }
}
