use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{protocol::identifiers::RequestId, string::CodepageString};

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Extended Message (like [Mst](super::Mst), but longer)
pub struct Msx {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[deku(bytes = "96")]
    pub msg: CodepageString,
}
