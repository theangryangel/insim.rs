use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{protocol::identifiers::RequestId, string::CodepageString};

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Insim Multiplayer - LFS sends this when a host is started or joined
pub struct Ism {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[deku(pad_bytes_after = "3")]
    pub host: u8,

    #[deku(bytes = "16")]
    pub hname: CodepageString,
}
