use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{protocol::identifiers::RequestId, string::istring};

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Version
pub struct Version {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[insim(
        reader = "istring::read(insim::rest, 8)",
        writer = "istring::write(insim::output, &self.version, 8)"
    )]
    pub version: String,

    #[insim(
        reader = "istring::read(insim::rest, 6)",
        writer = "istring::write(insim::output, &self.product, 6)"
    )]
    pub product: String,

    #[insim(pad_bytes_after = "1")]
    pub insimver: u8,
}
