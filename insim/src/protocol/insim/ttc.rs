use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::{ConnectionId, RequestId};

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum TtcType {
    None = 0,

    Selection = 1, // Send Axm for the current layout editor selection

    SelectionStart = 2, // Send Axm every time the selection changes

    SelectionStop = 3, // Stop sending Axm's
}

impl Default for TtcType {
    fn default() -> Self {
        TtcType::None
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// General purpose Target To Connection packet
pub struct Ttc {
    pub reqi: RequestId,

    pub subtype: TtcType,

    pub ucid: ConnectionId,

    pub b1: u8,

    pub b2: u8,
    
    pub b3: u8,
}
