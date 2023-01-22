use insim_core::prelude::*;

use crate::protocol::identifiers::{PlayerId, RequestId};
use super::CarContact;

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
/// Used within the [Csc] packet to indicate the type of state change.
pub enum CscAction {
    Stop = 0,

    Start = 1,
}

impl Default for CscAction {
    fn default() -> Self {
        CscAction::Stop
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Car State Changed
pub struct Csc {
    pub reqi: RequestId,

    #[deku(pad_bytes_after = "1")]
    pub plid: PlayerId,

    #[deku(pad_bytes_after = "2")]
    pub action: CscAction,

    pub time: u32,

    pub c: CarContact,
}
