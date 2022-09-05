use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::RequestId;

#[derive(Debug, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    type = "u8",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
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

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// General purpose Small packet
pub struct Small {
    pub reqi: RequestId,

    pub subtype: SmallType,

    pub uval: u32,
}
