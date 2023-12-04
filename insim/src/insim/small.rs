use insim_core::{
    binrw::{self, binrw},
    identifiers::RequestId,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum SmallType {
    #[default]
    None = 0,

    /// Request LFS to start sending positions
    Ssp = 1,

    /// Request LFS to start sending gauges
    Ssg = 2,

    /// Vote action
    Vta = 3,

    /// Time stop
    Tms = 4,

    /// Time step
    Stp = 5,

    /// Race time packet (reply to Gth)
    Rtp = 6,

    /// Set node lap interval
    Nli = 7,

    /// Set or get allowed cars (Tiny, type = Alc)
    Alc = 8,

    /// Set local car switches
    Lcs = 9,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// General purpose Small packet
pub struct Small {
    pub reqi: RequestId,

    pub subt: SmallType,

    pub uval: u32,
}
