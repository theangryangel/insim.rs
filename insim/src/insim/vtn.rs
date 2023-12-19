use insim_core::{
    binrw::{self, binrw},
    identifiers::{ConnectionId, RequestId},
};

/// Enum for the action field of [Vtn].
#[binrw]
#[derive(Default, Debug, Clone, Eq, PartialEq)]
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

// For usage in IS_SMALL
impl From<u32> for VtnAction {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::End,
            2 => Self::Restart,
            3 => Self::Qualify,
            _ => Self::None,
        }
    }
}

// For usage in IS_SMALL
impl From<&VtnAction> for u32 {
    fn from(value: &VtnAction) -> Self {
        match value {
            VtnAction::End => 1,
            VtnAction::Restart => 2,
            VtnAction::Qualify => 3,
            VtnAction::None => 0,
        }
    }
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
