use insim_core::binrw::{self, binrw};

use crate::identifiers::{ConnectionId, RequestId};

#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
/// Used within the [Cim] packet to indicate the mode.
pub enum CimMode {
    #[default]
    /// Not in a special mode
    Normal = 0,

    /// Options screen
    Options = 1,

    /// Host options screen
    HostOptions = 2,

    /// Garage screen
    Garage = 3,

    /// Vehicle select screen
    VehicleSelect = 4,

    /// Track select screen
    TrackSelect = 5,

    // Shift+U mode
    ShiftU = 6,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Connection Interface Mode
pub struct Cim {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// connection's unique id (0 = local)
    pub ucid: ConnectionId,

    /// Mode
    pub mode: CimMode,

    /// Submode
    // TODO: roll this into CimMode
    pub submode: u8,

    #[brw(pad_after = 1)]
    /// SelType is the selected object type or zero if unselected
    /// It may be an AXO_x as in ObjectInfo or one of these:
    /// const int MARSH_IS_CP		= 252; // insim checkpoint
    /// const int MARSH_IS_AREA		= 253; // insim circle
    /// const int MARSH_MARSHAL		= 254; // restricted area
    /// const int MARSH_ROUTE		= 255; // route checker
    pub seltype: u8,
}
