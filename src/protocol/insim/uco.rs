use super::{CarContact, ObjectInfo};
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(type = "u8", endian = "little")]
pub enum UcoAction {
    #[deku(id = "0")]
    Entered, // entered a circle

    #[deku(id = "1")]
    Left, // left a circle

    #[deku(id = "2")]
    CrossForwards, // crossed cp in forward direction

    #[deku(id = "3")]
    CrossedReverse,
}

impl Default for UcoAction {
    fn default() -> Self {
        UcoAction::Entered
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// User Control Object
pub struct Uco {
    pub reqi: u8,

    #[deku(pad_bytes_after = "1")]
    pub plid: u8,

    #[deku(pad_bytes_before = "2")]
    pub action: UcoAction,

    pub time: u32,

    pub c: CarContact,

    pub info: ObjectInfo,
}
