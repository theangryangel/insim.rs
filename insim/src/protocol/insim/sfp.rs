use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::RequestId;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// State Flags Pack
pub struct Sfp {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub flag: u16,

    #[insim(pad_bytes_after = "1")]
    pub onoff: u8,
}
