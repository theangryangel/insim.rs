use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{PlayerId, RequestId};

use super::{CarContact, ObjectInfo};

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum UcoAction {
    Entered = 0, // entered a circle

    Left = 1, // left a circle

    CrossForwards = 2, // crossed cp in forward direction

    CrossedReverse = 3,
}

impl Default for UcoAction {
    fn default() -> Self {
        UcoAction::Entered
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// User Control Object
pub struct Uco {
    pub reqi: RequestId,

    #[deku(pad_bytes_after = "1")]
    pub plid: PlayerId,

    #[deku(pad_bytes_before = "2")]
    pub action: UcoAction,

    pub time: u32,

    pub c: CarContact,

    pub info: ObjectInfo,
}
