use insim_core::{
    identifiers::{PlayerId, RequestId},
    binrw::{self, binrw}
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Player Tele-pits
pub struct Plp {
    pub reqi: RequestId,
    pub plid: PlayerId,
}
