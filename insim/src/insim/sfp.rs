use super::StaFlags;
use crate::identifiers::RequestId;

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Set a game state flag.
///
/// - Updates a single [StaFlags] value.
/// - Some states can only be toggled via text commands.
pub struct Sfp {
    /// Request identifier echoed by replies.
    #[insim(pad_after = 1)]
    pub reqi: RequestId,

    /// State flag to set or clear.
    pub flag: StaFlags,

    /// Whether to enable (true) or disable (false) the flag.
    #[insim(pad_after = 1)]
    pub onoff: bool,
}

impl_typical_with_request_id!(Sfp);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sfp() {
        assert_from_to_bytes!(
            Sfp,
            vec![
                0,   // ReqI
                0,   // Zero
                128, // Flag (1)
                4,   // Flag (2)
                1,   // OffOn
                0,   // Sp3
            ],
            |parsed: Sfp| {
                assert_eq!(parsed.reqi, RequestId(0));
                assert_eq!(parsed.onoff, true);
            }
        );
    }
}
