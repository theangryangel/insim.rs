use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
/// Used within the [Cim] packet to indicate the mode.
pub enum CimMode {
    #[default]
    Normal = 0,

    Options = 1,

    HostOptions = 2,

    Garage = 3,

    VehicleSelect = 4,

    TrackSelect = 5,

    ShiftU = 6,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Connection Interface Mode
pub struct Cim {
    pub reqi: RequestId,
    pub ucid: ConnectionId,

    pub mode: CimMode,
    pub submode: u8, // FIXME: How do we support this in the same way? LFS has multiple enum types.

    #[brw(pad_after = 1)]
    pub seltype: u8,
}
