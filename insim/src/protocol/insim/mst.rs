use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{protocol::identifiers::RequestId, string::CodepageString};

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Message Type - Send a message to LFS as if typed by a user
pub struct Mst {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[deku(bytes = "64")]
    pub msg: CodepageString,
}
