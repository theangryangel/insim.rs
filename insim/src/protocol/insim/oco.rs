use insim_core::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::protocol::identifiers::RequestId;

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum OcoAction {
    LightsReset = 4,

    LightsSet = 5,

    LightsUnset = 6,
}

impl Default for OcoAction {
    fn default() -> Self {
        OcoAction::LightsReset
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum OcoIndex {
    AxoStartLights = 149,

    MainLights = 240,
}

impl Default for OcoIndex {
    fn default() -> Self {
        OcoIndex::MainLights
    }
}

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct OcoLights: u8 {
        RED1 => (1 << 0),
        RED2 => (1 << 1),
        RED3 => (1 << 2),
        GREEN => (1 << 3),
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Object Control
/// Used to switch start lights
pub struct Oco {
    #[deku(pad_bytes_after = "1")]
    pub reqi: RequestId,

    pub action: OcoAction,

    pub index: OcoIndex,

    pub identifer: u8,

    pub lights: OcoLights,
}
