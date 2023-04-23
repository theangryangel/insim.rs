use std::time::Duration;

use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::{Fuel200, PenaltyInfo};

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Split timing
pub struct Spx {
    pub reqi: RequestId,
    pub plid: PlayerId,

    pub stime: Duration,
    pub etime: Duration,

    pub split: u8,
    pub penalty: PenaltyInfo,

    pub numstops: u8,
    pub fuel200: Fuel200,
}
