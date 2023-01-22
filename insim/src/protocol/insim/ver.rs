use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{protocol::identifiers::RequestId, string::istring};

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Version
pub struct Version {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[deku(
        reader = "istring::read(deku::rest, 8)",
        writer = "istring::write(deku::output, &self.version, 8)"
    )]
    pub version: String,

    #[deku(
        reader = "istring::read(deku::rest, 6)",
        writer = "istring::write(deku::output, &self.product, 6)"
    )]
    pub product: String,

    #[deku(pad_bytes_after = "1")]
    pub insimver: u8,
}
