use crate::protocol::identifiers::{PlayerId, RequestId};

use super::CarContact;
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
/// Used within the [Csc] packet to indicate the type of state change.
pub enum CscAction {
    #[deku(id = "0")]
    Stop,

    #[deku(id = "1")]
    Start,
}

impl Default for CscAction {
    fn default() -> Self {
        CscAction::Stop
    }
}

#[derive(Debug, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(
    ctx = "endian: deku::ctx::Endian",
    ctx_default = "deku::ctx::Endian::Little",
    endian = "endian"
)]
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
