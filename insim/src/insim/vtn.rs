use insim_core::binrw::{self, binrw};

use crate::identifiers::{ConnectionId, RequestId};

/// Enum for the action field of [Vtn].
#[binrw]
#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
#[non_exhaustive]
pub enum VtnAction {
    /// No vote, or cancel vote
    #[default]
    None = 0,

    /// Vote to end race
    End = 1,

    /// Vote to restart race
    Restart = 2,

    /// Vote to qualify
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

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Vote Notification
pub struct Vtn {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// The unique connection id of the connection that voted
    pub ucid: ConnectionId,

    /// The action or fact for this vote notification
    #[brw(pad_after = 2)]
    pub action: VtnAction,
}
