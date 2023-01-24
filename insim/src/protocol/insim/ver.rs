use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Version
pub struct Version {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[insim(bytes = "8")]
    pub version: String,

    #[insim(bytes = "6")]
    pub product: String,

    #[insim(pad_bytes_after = "1")]
    pub insimver: u8,
}
