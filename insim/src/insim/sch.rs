use bitflags::bitflags;
use insim_core::{identifiers::RequestId, binrw::{self, binrw}};

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// Bitwise flags used within the [Sch] packet
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    pub struct SchFlags: u8 {
        /// Shift
        const SHIFT = (1 << 0);

        /// Ctrl
        const CTRL = (1 << 1);
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Send Single Character
pub struct Sch {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    #[bw(map = |&x| x as u8)]
    #[br(map = |x: u8| x as char)]
    pub charb: char,

    #[brw(pad_after = 2)]
    pub flags: SchFlags,
}
