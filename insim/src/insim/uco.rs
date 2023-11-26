use std::time::Duration;

use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::{CarContact, ObjectInfo};

#[derive(Debug, Default, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum UcoAction {
    #[default]
    CircleEnter = 0, // entered a circle

    CircleLeave = 1, // left a circle

    CpFwd = 2, // crossed cp in forward direction

    CpRev = 3,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// User Control Object
pub struct Uco {
    pub reqi: RequestId,
    #[insim(pad_bytes_after = "1")]
    pub plid: PlayerId,

    #[insim(pad_bytes_after = "2")]
    pub action: UcoAction,

    pub time: Duration,

    pub c: CarContact,

    pub info: ObjectInfo,
}
