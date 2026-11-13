use crate::identifiers::RequestId;

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Screen mode configuration.
///
/// - Set resolution and refresh rate, or switch to windowed mode.
pub struct Mod {
    #[insim(pad_after = 1)]
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Set to choose 16-bit mode.
    pub bit16: i32,

    /// Refresh rate (0 = default).
    pub rr: i32,

    /// Screen width (0 = windowed mode).
    pub width: i32,

    /// Screen height (0 = windowed mode).
    pub height: i32,
}

impl_typical_with_request_id!(Mod);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mod() {
        let raw = [
            0,   // reqi
            0,   // zero
            2,   // bits16 (1)
            0,   // bits16 (2)
            0,   // bits16 (3)
            0,   // bits16 (4)
            59,  // rr (1)
            0,   // rr (2)
            0,   // rr (3)
            0,   // rr (4)
            128, // width (1)
            7,   // width (2)
            0,   // width (3)
            0,   // width (4)
            56,  // height (1)
            4,   // height (2)
            0,   // height (3)
            0,   // height (4)
        ];
        assert_from_to_bytes!(Mod, raw, |parsed: Mod| {
            assert_eq!(parsed.reqi, RequestId(0));
            assert_eq!(parsed.bit16, 2);
            assert_eq!(parsed.rr, 59);
            assert_eq!(parsed.width, 1920);
            assert_eq!(parsed.height, 1080);
        });
    }
}
