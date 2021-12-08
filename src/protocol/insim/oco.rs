use crate::packet_flags;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(type = "u8", endian = "little")]
pub enum OcoAction {
    #[deku(id = "4")]
    LightsReset,

    #[deku(id = "5")]
    LightsSet,

    #[deku(id = "6")]
    LightsUnset,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(type = "u8", endian = "little")]
pub enum OcoIndex {
    #[deku(id = "149")]
    AxoStartLights,

    #[deku(id = "240")]
    MainLights,
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

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Object Control
/// Used to switch start lights
pub struct Oco {
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    pub action: OcoAction,

    pub index: OcoIndex,

    pub identifer: u8,

    pub lights: OcoLights,
}
