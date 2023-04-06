use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// AutoX Object Contact
#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Axo {
    pub reqi: RequestId,
    pub plid: PlayerId,
}
