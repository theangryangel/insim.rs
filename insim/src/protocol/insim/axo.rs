use insim_core::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{PlayerId, RequestId};

/// AutoX Object Contact
#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Axo {
    pub reqi: RequestId,
    pub plid: PlayerId,
}
