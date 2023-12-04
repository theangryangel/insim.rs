use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, RequestId},
};

/// Enum for the action field of [Vtn].
#[binrw]
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
pub enum VtnAction {
    #[default]
    None = 0,

    End = 1,

    Restart = 2,

    Qualify = 3,
}

#[cfg(feature = "serde")]
use serde::Serialize;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Vote Notification
pub struct Vtn {
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    #[brw(pad_after = 2)]
    pub action: VtnAction,
}
