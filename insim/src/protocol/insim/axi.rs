use crate::{protocol::identifiers::RequestId, string::CodepageString};
use insim_core::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

/// Auto X Info
#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Axi {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub axstart: u8,
    pub numcp: u8,
    pub numo: u16,

    #[deku(bytes = "32")]
    pub lname: CodepageString,
}
