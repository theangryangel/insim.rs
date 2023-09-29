use std::time::Duration;

use insim_core::{
    identifiers::{PlayerId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::CarContact;

#[derive(Debug, Default, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
/// Used within the [Csc] packet to indicate the type of state change.
pub enum CscAction {
    #[default]
    Stop = 0,

    Start = 1,
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Car State Changed
pub struct Csc {
    pub reqi: RequestId,
    #[insim(pad_bytes_after = "1")]
    pub plid: PlayerId,

    #[insim(pad_bytes_after = "2")]
    pub action: CscAction,

    pub time: Duration,

    pub c: CarContact,
}
