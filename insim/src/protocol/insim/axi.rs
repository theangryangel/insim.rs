use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Auto X Info
#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Axi {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub axstart: u8,
    pub numcp: u8,
    pub numo: u16,

    #[insim(bytes = "32")]
    pub lname: String,
}
