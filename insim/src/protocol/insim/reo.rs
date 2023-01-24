use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Reorder
pub struct Reo {
    pub reqi: RequestId,

    pub nump: u8,

    #[insim(count = "40")]
    pub plid: Vec<PlayerId>,
}
