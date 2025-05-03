use bitflags::bitflags;

use crate::identifiers::RequestId;

bitflags! {
    /// Bitwise flags used within the [Sch] packet
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Send Single Character
pub struct Sch {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[read_write_buf(pad_after = 1)]
    pub reqi: RequestId,

    /// Character
    pub charb: char,

    /// Character modifiers/flags
    #[read_write_buf(pad_after = 2)]
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
