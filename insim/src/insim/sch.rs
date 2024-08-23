use bitflags::bitflags;
use insim_core::binrw::{self, binrw};

use crate::identifiers::RequestId;

bitflags! {
    /// Bitwise flags used within the [Sch] packet
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
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

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Send Single Character
pub struct Sch {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// Character
    #[bw(map = |&x| x as u8)]
    #[br(map = |x: u8| x as char)]
    pub charb: char,

    /// Character modifiers/flags
    #[brw(pad_after = 2)]
    pub flags: SchFlags,
}
