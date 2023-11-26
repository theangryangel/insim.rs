use insim_core::{
    identifiers::{ConnectionId, RequestId},
    prelude::*,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
/// Used within the [Cim] packet to indicate the mode.
pub enum CimMode {
    Normal = 0,

    Options = 1,

    HostOptions = 2,

    Garage = 3,

    VehicleSelect = 4,

    TrackSelect = 5,

    ShiftU = 6,
}

impl Default for CimMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, InsimEncode, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Connection Interface Mode
pub struct Cim {
    pub reqi: RequestId,
    pub ucid: ConnectionId,

    pub mode: CimMode,
    pub submode: u8, // FIXME: How do we support this in the same way? LFS has multiple enum types.
    #[insim(pad_bytes_after = "1")]
    pub seltype: u8,
}
