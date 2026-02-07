use bitflags::bitflags;

use crate::identifiers::RequestId;

bitflags! {
    /// Modifier flags used with [Sch].
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct SchFlags: u8 {
        /// Shift
        const SHIFT = (1 << 0);

        /// Ctrl
        const CTRL = (1 << 1);
    }
}

generate_bitflag_helpers! {
    SchFlags,
    pub shift => SHIFT,
    pub ctrl => CTRL
}

impl_bitflags_from_to_bytes!(SchFlags, u8);

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Send a single key press.
///
/// - Best for simple keys; some special keys may not work.
pub struct Sch {
    /// Request identifier echoed by replies.
    #[insim(pad_after = 1)]
    pub reqi: RequestId,

    /// Character to send.
    pub charb: char,

    /// Key modifiers (shift/ctrl).
    #[insim(pad_after = 2)]
    pub flags: SchFlags,
}

impl_typical_with_request_id!(Sch);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sch() {
        assert_from_to_bytes!(
            Sch,
            [
                1,  // ReqI
                0,  // Zero
                97, // CharB
                1,  // Flags
                0,  // Spare2
                0,  // Spare3
            ],
            |parsed: Sch| {
                assert_eq!(parsed.reqi, RequestId(1));
                assert_eq!(parsed.charb, 'a');
            }
        );
    }
}
