use insim_core::{
    identifiers::{PlayerId, RequestId},
    binrw::{self, binrw},
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Player Leaves race
pub struct Pll {
    pub reqi: RequestId,
    pub plid: PlayerId,
}
