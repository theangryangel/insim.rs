use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum SmallType {
    None = 0,

    Ssp = 1,

    Ssg = 2,

    Vta = 3,

    Tms = 4,

    Stp = 5,

    Rtp = 6,

    Nli = 7,

    Alc = 8,

    Lcs = 9,
}

impl Default for SmallType {
    fn default() -> Self {
        SmallType::None
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// General purpose Small packet
pub struct Small {
    pub reqi: RequestId,

    pub subtype: SmallType,

    pub uval: u32,
}
