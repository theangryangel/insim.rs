use crate::protocol::identifiers::ConnectionId;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(type = "u8", endian = "little")]
/// Used within the [Cim] packet to indicate the mode.
pub enum CimMode {
    #[deku(id = "0")]
    Normal,

    #[deku(id = "1")]
    Options,

    #[deku(id = "2")]
    HostOptions,

    #[deku(id = "3")]
    Garage,

    #[deku(id = "4")]
    VehicleSelect,

    #[deku(id = "5")]
    TrackSelect,

    #[deku(id = "6")]
    ShiftU,
}

impl Default for CimMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Connection Interface Mode
pub struct Cim {
    pub reqi: u8,

    pub ucid: ConnectionId,

    pub mode: CimMode,

    pub submode: u8, // FIXME: How do we support this in the same way? LFS has multiple enum types.

    #[deku(pad_bytes_after = "1")]
    pub seltype: u8,
}
