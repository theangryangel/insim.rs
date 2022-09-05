use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{
    packet_flags,
    protocol::identifiers::{PlayerId, RequestId},
};

packet_flags! {
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct ObhFlags: u8 {
        LAYOUT => (1 << 0),
        CAN_MOVE => (1 << 1),
        WAS_MOVING => (1 << 2),
        ON_SPOT => (1 << 3),
    }
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
pub struct CarContact {
    pub direction: u8,
    pub heading: u8,
    pub speed: u8,
    pub z: u8,
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
/// Object Hit
pub struct Obh {
    pub reqi: RequestId,
    pub plid: PlayerId,

    pub spclose: u16,
    pub time: u16,

    pub info: CarContact,

    pub x: i16,
    pub y: i16,

    #[deku(pad_bytes_after = "1")]
    pub z: u8,
    pub index: u8,

    pub flags: ObhFlags,
}
