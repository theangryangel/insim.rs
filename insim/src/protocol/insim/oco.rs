use crate::{packet_flags, protocol::identifiers::RequestId};
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    type = "u8",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub enum OcoAction {
    #[deku(id = "4")]
    LightsReset,

    #[deku(id = "5")]
    LightsSet,

    #[deku(id = "6")]
    LightsUnset,
}

impl Default for OcoAction {
    fn default() -> Self {
        OcoAction::LightsReset
    }
}

#[derive(Debug, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    type = "u8",
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub enum OcoIndex {
    #[deku(id = "149")]
    AxoStartLights,

    #[deku(id = "240")]
    MainLights,
}

impl Default for OcoIndex {
    fn default() -> Self {
        OcoIndex::MainLights
    }
}

packet_flags! {
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct OcoLights: u8 {
        RED1 => (1 << 0),
        RED2 => (1 << 1),
        RED3 => (1 << 2),
        GREEN => (1 << 3),
    }
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
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
