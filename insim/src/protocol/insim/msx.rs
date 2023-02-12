use insim_core::{identifiers::RequestId, prelude::*};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Extended Message (like [Mst](super::Mst), but longer)
pub struct Msx {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[insim(bytes = "96")]
    pub msg: String,
}
