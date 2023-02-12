use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Insim Multiplayer - LFS sends this when a host is started or joined
pub struct Ism {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[insim(pad_bytes_after = "3")]
    pub host: u8,

    #[insim(bytes = "16")]
    pub hname: String,
}
