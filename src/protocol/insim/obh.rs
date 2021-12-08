use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::packet_flags;

packet_flags! {
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct ObhFlags: u8 {
        LAYOUT => (1 << 0),
        CAN_MOVE => (1 << 1),
        WAS_MOVING => (1 << 2),
        ON_SPOT => (1 << 3),
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(endian = "little")]
pub struct CarContact {
    pub direction: u8,
    pub heading: u8,
    pub speed: u8,
    pub z: u8,
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Object Hit
pub struct Obh {
    pub reqi: u8,
    pub plid: u8,

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
