use insim_core::{
    binrw::{self, binrw},
    identifiers::{PlayerId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// AutoX Object Contact
#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Axo {
    pub reqi: RequestId,
    pub plid: PlayerId,
}
