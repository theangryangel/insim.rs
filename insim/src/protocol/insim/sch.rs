use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Send Single Character
pub struct Sch {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub charb: u8,

    #[insim(pad_bytes_after = "2")]
    pub flags: u8, // bit 0: SHIFT / bit 1: CTRL
}
