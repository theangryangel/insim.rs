use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(type = "u8", endian = "little")]
pub enum SmallType {
    #[deku(id = "0")]
    None,

    #[deku(id = "1")]
    Ssp,

    #[deku(id = "2")]
    Ssg,

    #[deku(id = "3")]
    Vta,

    #[deku(id = "4")]
    Tms,

    #[deku(id = "5")]
    Stp,

    #[deku(id = "6")]
    Rtp,

    #[deku(id = "7")]
    Nli,

    #[deku(id = "8")]
    Alc,

    #[deku(id = "9")]
    Lcs,
}

impl Default for SmallType {
    fn default() -> Self {
        SmallType::None
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// General purpose Small packet
pub struct Small {
    #[deku(bytes = "1")]
    pub reqi: u8,

    pub subtype: SmallType,

    #[deku(bytes = "4")]
    pub uval: u32,
}
