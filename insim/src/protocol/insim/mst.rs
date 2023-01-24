use insim_core::{identifiers::RequestId, prelude::*, string::CodepageString};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Message Type - Send a message to LFS as if typed by a user
pub struct Mst {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[insim(bytes = "64")]
    pub msg: CodepageString,
}
